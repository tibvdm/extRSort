use std::io::Read;

use crate::Configuration;

use super::chunk::{Chunk, LINE_OVERHEAD, MAX_CHUNK_SIZE};

pub struct Chunks<R: Read> {
    input: R,
    carry_over: Vec<u8>,

    /// The memory for one chunk: its buffer and the offsets of its lines
    memory: usize,

    /// The average line length in the last chunk. It starts at the shortest line, a single
    /// newline, so the first chunk stays within memory whatever its lines are.
    bytes_per_line: usize,

    config: Configuration
}

impl<R: Read> Chunks<R> {
    pub fn new(input: R, memory: usize, config: Configuration) -> Self {
        Chunks {
            input,
            carry_over: vec![],
            memory,
            bytes_per_line: 1,
            config
        }
    }

    /// The buffer that leaves room in memory for the offsets of its lines
    fn buffer_size(&self) -> usize {
        (self.memory / (self.bytes_per_line + LINE_OVERHEAD) * self.bytes_per_line).clamp(1, MAX_CHUNK_SIZE)
    }
}

impl<R: Read> Iterator for Chunks<R> {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        let buffer_size = self.buffer_size();
        let chunk = Chunk::read(&mut self.input, &mut self.carry_over, buffer_size, &self.config)?;
        self.bytes_per_line = chunk.bytes_per_line();
        Some(chunk)
    }
}

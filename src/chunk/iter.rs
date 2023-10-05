use std::io::Read;

use crate::Configuration;

use super::chunk::Chunk;

pub struct Chunks<R: Read> {
    input: R,
    carry_over: Vec<u8>,
    buffer_size: usize,
    config: Configuration
}

impl<R: Read> Chunks<R> {
    pub fn new(input: R, buffer_size: usize, config: Configuration) -> Self {
        Chunks {
            input,
            carry_over: vec![],
            buffer_size,
            config
        }
    }
}

impl<R: Read> Iterator for Chunks<R> {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        Chunk::read(&mut self.input, &mut self.carry_over, self.buffer_size, &self.config)
    }
}

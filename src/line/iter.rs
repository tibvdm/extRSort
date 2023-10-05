use std::io::Read;

use crate::{chunk::{Chunks, Chunk}, Configuration};

use super::Line;

pub struct Lines<R: Read> {
    chunks: Chunks<R>,
    chunk: Option<Chunk>
}

impl<R: Read> Lines<R> {
    pub fn new(input: R, buffer_size: usize, config: Configuration) -> Self {
        let mut chunks = Chunks::new(input, buffer_size, config);
        let chunk = chunks.next();

        Lines { chunks, chunk }
    }
}

impl<R: Read> Iterator for Lines<R> {
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(chunk) = &mut self.chunk {
            if let Some(line) = chunk.next() {
                return Some(line);
            }
        }

        if let Some(next_chunk) = self.chunks.next() {
            self.chunk = Some(next_chunk);
            return self.next();
        } 

        None
    }
}

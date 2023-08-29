use std::io::Read;

use super::chunk::Chunk;

pub struct Chunks<R: Read> {
    input: R,
    carry_over: Vec<u8>,
    buffer_size: usize
}

impl<R: Read> Chunks<R> {
    pub fn new(input: R, buffer_size: usize) -> Self {
        Chunks {
            input,
            carry_over: vec![],
            buffer_size
        }
    }
}

impl<R: Read> Iterator for Chunks<R> {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        Chunk::read(&mut self.input, &mut self.carry_over, self.buffer_size)
    }
}

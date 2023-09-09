use std::{io::Read, fmt::Debug};

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

impl<R: Read> Debug for Chunks<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunks").finish()
    }
}

impl<R: Read> Iterator for Chunks<R> {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        Chunk::read(&mut self.input, &mut self.carry_over, self.buffer_size)
    }
}

impl<R: Read> PartialEq for Chunks<R> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<R: Read> PartialOrd for Chunks<R> {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ordering::Equal)
    }   
}

impl<R: Read> Eq for Chunks<R> {}

impl<R: Read> Ord for Chunks<R> {
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}

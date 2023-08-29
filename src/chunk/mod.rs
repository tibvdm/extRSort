mod chunk;
mod iter;

use std::cmp::Ordering::Less;
use std::io::Write;
pub use chunk::Chunk;
pub use iter::Chunks;

use crate::line::Line;

pub struct ChunkLine {
    pub chunk: Chunk,
    pub line_len: usize,
    pub line_index: usize,
    pub iterator_index: usize
}

impl ChunkLine {
    pub fn new(chunk: Chunk, line_index: usize, iterator_index: usize) -> Self {
        let line_len = chunk.line(line_index).len();
        ChunkLine { chunk, line_len, line_index, iterator_index }
    }

    pub fn write(&self, writer: &mut impl Write) {
        self.chunk.line(self.line_index).write(writer);
    }
}

impl PartialEq for ChunkLine {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl PartialOrd for ChunkLine {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for ChunkLine {}

impl Ord for ChunkLine {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if other.line_len < self.line_len {
            return Less;
        }

        other.chunk.line(other.line_index).cmp(&self.chunk.line(self.line_index))
    }
}

mod chunk;
mod iter;

use std::io::Write;
pub use chunk::Chunk;
pub use iter::Chunks;

pub struct ChunkLine {
    pub chunk: Chunk,
    pub line_index: usize,
    pub iterator_index: usize
}

impl ChunkLine {
    pub fn new(chunk: Chunk, line_index: usize, iterator_index: usize) -> Self {
        ChunkLine { chunk, line_index, iterator_index }
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
        other.chunk.line(other.line_index).cmp(&self.chunk.line(self.line_index))
    }
}

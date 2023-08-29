mod chunk;
mod iter;

pub use chunk::Chunk;
pub use iter::Chunks;

use crate::line::Line;

pub struct ChunkLine {
    pub chunk: Option<Chunk>,
    pub line_index: usize,
    pub iterator_index: usize
}

impl ChunkLine {
    pub fn new(chunk: Option<Chunk>, line_index: usize, iterator_index: usize) -> Self {
        ChunkLine { chunk, line_index, iterator_index }
    }

    pub fn current_line(&self) -> Option<&Line> {
        if let Some(chunk) = self.chunk.as_ref() {
            return chunk.lines().get(self.line_index);
        }

        None
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
        return other.chunk.as_ref().unwrap().lines().get(other.line_index).cmp(
            &self.chunk.as_ref().unwrap().lines().get(self.line_index)
        )
    }
}

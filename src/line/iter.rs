use std::io::Read;

use crate::{chunk::{Chunks, Chunk}, Configuration};

use super::Line;

/// Iterator over the lines of a file
pub struct Lines<R: Read> {
    /// Chunked iterator over the lines of a file
    chunks: Chunks<R>,

    /// The current chunk
    chunk: Option<Chunk>
}

impl<R: Read> Lines<R> {
    /// Creates a new iterator over the lines of a file
    /// 
    /// # Arguments
    /// 
    /// * `input` - The input to read from
    /// * `buffer_size` - The buffer size per chunk
    /// * `config` - Some additional configuration options
    /// 
    /// # Returns
    /// 
    /// A new (chunked) iterator over the lines of a file
    pub fn new(input: R, buffer_size: usize, config: Configuration) -> Self {
        let mut chunks = Chunks::new(input, buffer_size, config);
        let chunk = chunks.next();

        Lines { chunks, chunk }
    }
}

impl<R: Read> Iterator for Lines<R> {
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        // If there is a chunk, try to get the next line from it
        if let Some(chunk) = &mut self.chunk {
            if let Some(line) = chunk.next() {
                return Some(line);
            }
        }

        // If there is no chunk, try to get the next chunk
        if let Some(next_chunk) = self.chunks.next() {
            self.chunk = Some(next_chunk);
            return self.next();
        }

        // If there are no chunks left, return None
        None
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;

    fn construct_rc_buffer(content: &str) -> Rc<Vec<u8>> {
        Rc::new(content.as_bytes().to_vec())
    }

    #[test]
    fn test_new_one_chunk() {
        let input_vec = "AAACLNNYAA\nAAAAAALTER\nKYYLMMAAFG\n".as_bytes().to_vec();
        let reader = std::io::Cursor::new(input_vec);

        let mut lines = Lines::new(reader, 100, Configuration::default());

        let buffer = construct_rc_buffer("AAACLNNYAA\nAAAAAALTER\nKYYLMMAAFG\n");

        assert_eq!(lines.next(), Some(Line::new(buffer.clone(), 0, 9)));
        assert_eq!(lines.next(), Some(Line::new(buffer.clone(), 11, 20)));
        assert_eq!(lines.next(), Some(Line::new(buffer.clone(), 22, 31)));
        assert_eq!(lines.next(), None);
    }

    #[test]
    fn test_new_multiple_chunks() {
        let input_vec = "AAACLNNYAA\nAAAAAALTER\nKYYLMMAAFG\n".as_bytes().to_vec();
        let reader = std::io::Cursor::new(input_vec);

        let mut lines = Lines::new(reader, 15, Configuration::default());

        let buffer = construct_rc_buffer("AAACLNNYAA\nAAAAAALTER\nKYYLMMAAFG\n");

        assert_eq!(lines.next(), Some(Line::new(buffer.clone(), 0, 9)));
        assert_eq!(lines.next(), Some(Line::new(buffer.clone(), 11, 20)));
        assert_eq!(lines.next(), Some(Line::new(buffer.clone(), 22, 31)));
        assert_eq!(lines.next(), None);
    }
}

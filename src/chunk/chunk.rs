use std::{io::{Read, Write}, str::from_utf8};

use memchr::memrchr_iter;
use self_cell::self_cell;

use crate::line::{self, Line};

// We need a self referencing cell because a Line is just a reference
// to a &str, which in turn is a reference to the owner/buffer.
self_cell!(
    /// The chunk that is passed around between threads.
    pub struct Chunk {
        // {owner} is the buffer that holds all read data
        owner: Vec<u8>,

        // All processed data/slices from the owner
        #[covariant]
        dependent: ChunkData,
    }

    impl { Debug }
);

#[derive(Debug)]
pub struct ChunkData<'a> {
    pub lines: Vec<Line<'a>>,
    pub current_line: usize
}

impl Chunk {
    pub fn read<R: Read>(
        input: &mut R, 
        carry_over: &mut Vec<u8>,
        buffer_size: usize
    ) -> Option<Self> {
        let mut buffer = vec![0; buffer_size];
    
        // Put the carry over bytes at the beginning of the buffer
        buffer[..carry_over.len()].copy_from_slice(carry_over);
    
        // Fill the buffer with the next input bytes
        let (completed, bytes_read) = fill_buffer(input, &mut buffer, carry_over.len());
    
        // Move the carry over bytes from the end of the buffer to the carry over vector
        carry_over.clear();
        if !completed {
            carry_over.extend_from_slice(&buffer[bytes_read..]);
        }
    
        // If we read some new bytes
        if bytes_read != 0 {
            let chunk = Chunk::new(buffer, |buffer| {
                let lines_str = from_utf8(&buffer[..bytes_read]).unwrap();    
                let lines = line::lines(lines_str).collect();
    
                ChunkData { lines, current_line: 0 }
            });
    
            return Some(chunk);
        }
    
        None
    }

    pub fn write<W: Write>(&self, writer: &mut W) {
        for line in self.borrow_dependent().lines.iter() {
            line.write(writer);
        }
    }

    pub fn line(&self, index: usize) -> &Line {
        self.borrow_dependent().lines.get(index).unwrap()
    }

    pub fn len(&self) -> usize {
        self.borrow_dependent().lines.len()
    }

    pub fn next_line(&mut self) -> &Line {
        self.with_dependent_mut(|_, data| {
            let line = &data.lines[data.current_line];
            data.current_line += 1;
            line
        })
    }

    pub fn is_empty(&self) -> bool {
        self.borrow_dependent().current_line >= self.len()
    }
}

impl PartialEq for Chunk {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl PartialOrd for Chunk {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }   
}

impl Eq for Chunk {}

impl Ord for Chunk {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.line(0).cmp(other.line(0))
    }   
}

fn fill_buffer<T: Read>(
    input: &mut T,
    buffer: &mut Vec<u8>,
    offset: usize
) -> (bool, usize) {
    // Store the buffer size in advance, because rust will complain 
    // about the buffer being borrowed mutably while it's borrowed
    let buffer_size = buffer.len();

    // Skip the first {offset} bytes that we still need to read from 
    // the previous read operation
    let mut writable_buffer_space = buffer[offset..].as_mut();

    loop {
        match input.read(writable_buffer_space) {
            // No bytes written, which means we've completely filled the buffer
            // or we've reached the end of the file
            Ok(0) => {
                // No bytes written and the buffer slice has size zero. This means 
                // that we've completely filled the buffer
                if writable_buffer_space.is_empty() {
                    // Create a very optimized reversed iterator over the newlines
                    let mut lines_reversed = memrchr_iter(b'\n', buffer);

                    // The last line is incomplete, so we only report the number 
                    // of bytes that we've read till that last line. We add 1
                    // because we don't want to keep the newline.
                    let bytes_read = lines_reversed.next().unwrap() + 1;

                    // Return the number of bytes that we read
                    return (false, bytes_read);
                }

                // No bytes written and a non-empty buffer indicates that we've 
                // reached the end of the file
                else {
                    let bytes_read = buffer_size - writable_buffer_space.len();

                    // Return the number of bytes that we read
                    return (true, bytes_read);
                }
            },

            // We've read {bytes_read} bytes
            Ok(bytes_read) => {
                // Shrink the writable buffer slice
                writable_buffer_space = writable_buffer_space[bytes_read..].as_mut();
            },

            Err(err) => {
                panic!("Error while reading input: {}", err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use crate::line;

    use super::{Chunk, ChunkData};

    fn new_chunk(amount_of_lines: usize) -> Chunk {
        let buffer = (1..=amount_of_lines).map(|i| format!("line{}\n", i)).collect::<String>().as_bytes().to_vec();

        Chunk::new(buffer, |buffer| {
            let lines_str = from_utf8(&buffer).unwrap();    
            let lines = line::lines(lines_str).collect();

            ChunkData { lines, current_line: 0 }
        })
    }

    #[test]
    fn test_chunk_next_line() {
        let mut chunk = new_chunk(3);

        assert_eq!(chunk.next_line().content, "line1");
        assert_eq!(chunk.next_line().content, "line2");
        assert_eq!(chunk.next_line().content, "line3");
    }
}

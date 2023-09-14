use std::{io::{Read, Write}, rc::Rc, fmt::Debug};

use memchr::{memrchr_iter, memchr_iter};

use crate::line::Line;

pub struct Chunk {
    lines: Vec<Line>,
    current_line: usize
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

        let buffer = Rc::new(buffer);
    
        // If we read some new bytes
        if bytes_read != 0 {
            let mut start_index = 0;
            let mut lines = Vec::with_capacity(bytes_read);
            for end_index in memchr_iter(b'\n', &buffer[..bytes_read]) {
                // End index includes the newline
                lines.push(Line::new(Rc::clone(&buffer), start_index, end_index - 1));
                start_index = end_index + 1;
            }

            return Some(Chunk { lines, current_line: 0 });
        }
    
        None
    }

    pub fn write<W: Write>(&self, writer: &mut W) {
        for line in &self.lines {
            line.write(writer);
        }
    }

    pub fn sort_unstable(&mut self) {
        self.lines.sort_unstable_by(|a, b| b.cmp(a));
    }
}

impl Iterator for Chunk {
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(line) = self.lines.get(self.current_line) {
            self.current_line += 1;
            return Some(line.clone());
        }

        None
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("line1: ", &self.lines[0])
            .field("line2: ", &self.lines[1])
            // .field("line3: ", &self.lines[2])
            // .field("line4: ", &self.lines[3])
            // .field("line5: ", &self.lines[4])
            .finish()
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
        other.lines[other.current_line].cmp(&self.lines[self.current_line])
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

// #[cfg(test)]
// mod tests {
//     use std::rc::Rc;

//     use crate::{line::Line, chunk::Chunk};

//     const BUFFER_STRING: &str = "AAAALTER\nAAA\nAAAA\nAAAALTER\nAAAALTERRR\nCAAAALTER\n";

//     fn new_chunk() -> Chunk {
//         let buffer = Rc::new(BUFFER_STRING.as_bytes().to_vec());

//         let line1 = Line::new(Rc::clone(&buffer), 0, 7);
//         let line2 = Line::new(Rc::clone(&buffer), 9, 11);
//         let line3 = Line::new(Rc::clone(&buffer), 13, 16);
//         let line4 = Line::new(Rc::clone(&buffer), 18, 25);
//         let line5 = Line::new(Rc::clone(&buffer), 27, 36);
//         let line6 = Line::new(Rc::clone(&buffer), 38, 46);

//         Chunk { lines: vec![ line1, line2, line3, line4, line5, line6 ], current_line: 0 }
//     }

//     #[test]
//     fn test_chunk_read_sufficient_buffer() {
//         // TODO: add a test where a line is too big to fit in the buffer
//         let mut carry_over = vec![];
//         let mut input = BUFFER_STRING.as_bytes();

//         let chunk = Chunk::read(&mut input, &mut carry_over, 32).unwrap();
//         assert_eq!(chunk.lines.len(), 4);
//         assert_eq!(chunk.lines[0].as_bytes(), "AAAALTER".as_bytes());
//         assert_eq!(chunk.lines[1].as_bytes(), "AAA".as_bytes());
//         assert_eq!(chunk.lines[2].as_bytes(), "AAAA".as_bytes());
//         assert_eq!(chunk.lines[3].as_bytes(), "AAAALTER".as_bytes());

//         let chunk = Chunk::read(&mut input, &mut carry_over, 32).unwrap();
//         assert_eq!(chunk.lines.len(), 2);
//         assert_eq!(chunk.lines[0].as_bytes(), "AAAALTERRR".as_bytes());
//         assert_eq!(chunk.lines[1].as_bytes(), "CAAAALTER".as_bytes());

//         assert!(carry_over.is_empty());
//     }

//     #[test]
//     fn test_chunk_read_insufficient_buffer() {
//         unimplemented!()
//     }

//     #[test]
//     fn test_chunk_len() {
//         let chunk = new_chunk();
//         assert_eq!(chunk.len(), 6);
//     }

//     #[test]
//     fn test_chunk_next_line() {
//         let mut chunk = new_chunk();

//         assert_eq!(chunk.next_line().as_bytes(), "AAAALTER".as_bytes());
//         assert_eq!(chunk.next_line().as_bytes(), "AAA".as_bytes());
//         assert_eq!(chunk.next_line().as_bytes(), "AAAA".as_bytes());
//         assert_eq!(chunk.next_line().as_bytes(), "AAAALTER".as_bytes());
//         assert_eq!(chunk.next_line().as_bytes(), "AAAALTERRR".as_bytes());
//         assert_eq!(chunk.next_line().as_bytes(), "CAAAALTER".as_bytes());
//     }

//     #[test]
//     fn test_chunk_is_empty() {
//         let non_empty_chunk = new_chunk();
//         assert_eq!(non_empty_chunk.is_empty(), false);

//         let empty_chunk = Chunk { lines: vec![], current_line: 0 };
//         assert_eq!(empty_chunk.is_empty(), true);
//     }

//     #[test]
//     fn test_chunk_sort_unstable() {
//         let mut chunk = new_chunk();

//         chunk.sort_unstable();

//         assert_eq!(chunk.lines[0].as_bytes(), "AAA".as_bytes());
//         assert_eq!(chunk.lines[1].as_bytes(), "AAAA".as_bytes());
//         assert_eq!(chunk.lines[2].as_bytes(), "AAAALTER".as_bytes());
//         assert_eq!(chunk.lines[3].as_bytes(), "AAAALTER".as_bytes());
//         assert_eq!(chunk.lines[4].as_bytes(), "AAAALTERRR".as_bytes());
//         assert_eq!(chunk.lines[5].as_bytes(), "CAAAALTER".as_bytes());
//     }

//     // #[test]
//     // fn test_chunk_cmp() {
//     //     let buffer = Rc::new(BUFFER_STRING.as_bytes().to_vec());

//     //     let line1 = Line::new(Rc::clone(&buffer), 0, 8);
//     //     let line2 = Line::new(Rc::clone(&buffer), 9, 12);
//     //     let line3 = Line::new(Rc::clone(&buffer), 13, 17);
//     //     let line4 = Line::new(Rc::clone(&buffer), 18, 26);
//     //     let line5 = Line::new(Rc::clone(&buffer), 27, 37);
//     //     let line6 = Line::new(Rc::clone(&buffer), 38, 47);

//     //     let mut chunk1 = Chunk { lines: vec![ line1, line2, line3 ], current_line: 0 };
//     //     let mut chunk2 = Chunk { lines: vec![ line4, line5, line6 ], current_line: 0 };

//     //     assert_eq!(chunk1.cmp(&chunk2), std::cmp::Ordering::Equal);
//     //     assert_eq!(chunk2.cmp(&chunk1), std::cmp::Ordering::Equal);

//     //     chunk1.next();

//     //     assert_eq!(chunk1.cmp(&chunk2), std::cmp::Ordering::Less);
//     //     assert_eq!(chunk2.cmp(&chunk1), std::cmp::Ordering::Greater);

//     //     chunk1.next();
//     //     chunk2.next();
//     //     chunk2.next();

//     //     assert_eq!(chunk1.cmp(&chunk2), std::cmp::Ordering::Less);
//     //     assert_eq!(chunk2.cmp(&chunk1), std::cmp::Ordering::Greater);
//     // }
// }

// // #[cfg(test)]
// // mod tests {
// //     use std::str::from_utf8;

// //     use crate::line;

// //     use super::{Chunk, ChunkData};

// //     fn new_chunk(amount_of_lines: usize) -> Chunk {
// //         let buffer = (1..=amount_of_lines).map(|i| format!("line{}\n", i)).collect::<String>().as_bytes().to_vec();

// //         Chunk::new(buffer, |buffer| {
// //             let lines_str = from_utf8(&buffer).unwrap();    
// //             let lines = line::lines(lines_str).collect();

// //             ChunkData { lines, current_line: 0 }
// //         })
// //     }

// //     #[test]
// //     fn test_chunk_next_line() {
// //         let mut chunk = new_chunk(3);

// //         assert_eq!(chunk.next_line().content, "line1");
// //         assert_eq!(chunk.next_line().content, "line2");
// //         assert_eq!(chunk.next_line().content, "line3");
// //     }
// // }

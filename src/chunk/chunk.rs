use std::{io::{ErrorKind, Read, Write}, rc::Rc};

use memchr::{memchr_iter, memrchr};

use crate::{line::Line, Configuration};

pub struct Chunk {
    lines: Vec<Line>,
    current_line: usize
}

impl Chunk {
    pub fn read<R: Read>(
        input: &mut R, 
        carry_over: &mut Vec<u8>,
        buffer_size: usize,
        config: &Configuration
    ) -> Option<Self> {
        // At least as large as the carry over, which is longer than buffer_size after a long line
        let mut buffer = vec![0; buffer_size.max(carry_over.len())];
    
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

            // Adds the line from start_index up to end_index, which is the newline or the end of the input
            let mut push_line = |start_index: usize, end_index: usize| {
                if config.has_field() {
                    let offset = memchr_iter(config.delimiter, &buffer[start_index..end_index])
                        .skip(config.field - 2)
                        .next()
                        .expect("Fields should be correct");

                    lines.push(Line::new_with_field(Rc::clone(&buffer), start_index, end_index - 1, (start_index + offset, end_index - 1)));
                } else {
                    lines.push(Line::new(Rc::clone(&buffer), start_index, end_index - 1));
                }
            };

            for end_index in memchr_iter(b'\n', &buffer[..bytes_read]) {
                push_line(start_index, end_index);

                // End index includes the newline
                start_index = end_index + 1;
            }

            // The last line of the input, when it does not end in a newline
            if completed && start_index < bytes_read {
                push_line(start_index, bytes_read);
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

fn fill_buffer<T: Read>(
    input: &mut T,
    buffer: &mut Vec<u8>,
    offset: usize
) -> (bool, usize) {
    // Skip the first {offset} bytes that we still need to read from
    // the previous read operation
    let mut filled = offset;

    loop {
        // The buffer is full
        if filled == buffer.len() {
            match memrchr(b'\n', buffer) {
                // The last line is incomplete, so we only report the number
                // of bytes that we've read till that last line. We add 1
                // because we don't want to keep the newline.
                Some(last_newline) => return (false, last_newline + 1),

                // A line longer than the buffer: grow the buffer until the line fits
                None => buffer.resize(buffer.len().max(1) * 2, 0)
            }
        }

        match input.read(&mut buffer[filled..]) {
            // No bytes read in a buffer with space left: the end of the input
            Ok(0) => return (true, filled),

            // We've read {bytes_read} bytes
            Ok(bytes_read) => filled += bytes_read,

            Err(err) if err.kind() == ErrorKind::Interrupted => {},

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

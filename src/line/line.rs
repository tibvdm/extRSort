use std::{rc::Rc, io::Write};

/// A struct representing a single line of bytes
#[derive(Clone, Debug)]
pub struct Line {
    /// The buffer containing the bytes of the line
    /// We use a Reference counting smart pointer here to avoid copying the buffer for each individual line
    buffer: Rc<Vec<u8>>,

    /// The index of the first byte of this line in the buffer
    start: usize,

    /// The index of the last byte of this line in the buffer
    end: usize,

    /// The range of bytes in the line that should be used for sorting
    field: (usize, usize),
}

impl Line {
    /// Creates a new `Line` instance with the given buffer, start and end indices.
    /// The `field` attribute is set to the range between `start` and `end`.
    /// 
    /// # Arguments
    /// 
    /// * `buffer` - A smart pointer to the buffer containing the bytes of the line
    /// * `start` - The index of the first byte of this line in the buffer
    /// * `end` - The index of the last byte of this line in the buffer
    /// 
    /// # Returns
    /// 
    /// A new `Line` instance
    pub fn new(buffer: Rc<Vec<u8>>, start: usize, end: usize) -> Self {
        Line { buffer, start, end, field: (start, end) }
    }

    /// Creates a new `Line` instance with the given buffer, start and end indices and field range.
    /// 
    /// # Arguments
    /// 
    /// * `buffer` - A smart pointer to the buffer containing the bytes of the line
    /// * `start` - The index of the first byte of this line in the buffer
    /// * `end` - The index of the last byte of this line in the buffer
    /// * `field` - The range of bytes in the line that should be used for sorting
    /// 
    /// # Returns
    /// 
    /// A new `Line` instance
    pub fn new_with_field(buffer: Rc<Vec<u8>>, start: usize, end: usize, field: (usize, usize)) -> Self {
        Line { buffer, start, end, field }
    }

    /// Writes the line to the given writer
    /// 
    /// # Arguments
    /// 
    /// * `writer` - The writer to write the line to
    pub fn write(&self, writer: &mut impl Write) {
        writer.write_all(self.as_bytes()).unwrap();
        writer.write("\n".as_bytes()).unwrap();
    }

    /// Returns the bytes of the line
    fn as_bytes(&self) -> &[u8] {
        &self.buffer[self.start..=self.end]
    }

    /// Returns the bytes of the line that should be used for sorting
    fn as_sort_bytes(&self) -> &[u8] {
        &self.buffer[self.field.0..=self.field.1]
    }
}

unsafe impl Send for Line {}

impl PartialEq for Line {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl Eq for Line {}

impl PartialOrd for Line {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.as_sort_bytes().partial_cmp(self.as_sort_bytes())
    }
}

impl Ord for Line {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.as_sort_bytes().cmp(self.as_sort_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn construct_rc_buffer(content: &str) -> Rc<Vec<u8>> {
        Rc::new(content.as_bytes().to_vec())
    }

    #[test]
    fn test_new() {
        let buffer = construct_rc_buffer("AAACLNNYAA");

        let line = Line::new(Rc::clone(&buffer), 0, 9);

        assert_eq!(line.as_bytes(), "AAACLNNYAA".as_bytes());
    }

    #[test]
    fn test_new_with_field() {
        let buffer = construct_rc_buffer("AAACLNNYAA");

        let line = Line::new_with_field(Rc::clone(&buffer), 0, 9, (1, 8));

        assert_eq!(line.as_bytes(), "AAACLNNYAA".as_bytes());
        assert_eq!(line.as_sort_bytes(), "AACLNNYA".as_bytes());
    }

    #[test]
    fn test_write() {
        let buffer = construct_rc_buffer("AAACLNNYAA");

        let line = Line::new(Rc::clone(&buffer), 0, 9);

        let mut output = vec![];
        line.write(&mut output);

        assert_eq!(output, "AAACLNNYAA\n".as_bytes());
    }

    #[test]
    fn test_as_bytes() {
        let buffer = construct_rc_buffer("AAACLNNYAA");

        let line = Line::new(Rc::clone(&buffer), 0, 9);
        let line_with_field = Line::new_with_field(Rc::clone(&buffer), 0, 9, (1, 8));

        assert_eq!(line.as_bytes(), "AAACLNNYAA".as_bytes());
        assert_eq!(line_with_field.as_bytes(), "AAACLNNYAA".as_bytes());
    }

    #[test]
    fn test_as_sort_bytes() {
        let buffer = construct_rc_buffer("AAACLNNYAA");

        let line = Line::new(Rc::clone(&buffer), 0, 9);
        let line_with_field = Line::new_with_field(Rc::clone(&buffer), 0, 9, (1, 8));

        assert_eq!(line.as_sort_bytes(), "AAACLNNYAA".as_bytes());
        assert_eq!(line_with_field.as_sort_bytes(), "AACLNNYA".as_bytes());
    }

    #[test]
    fn test_cmp() {
        let buffer = construct_rc_buffer("AAACL\nAAA\nCAAALTER\nAAA\n");

        let line1 = Line::new(Rc::clone(&buffer), 0, 4);
        let line2 = Line::new(Rc::clone(&buffer), 6, 8);
        let line3 = Line::new(Rc::clone(&buffer), 10, 17);
        let line4 = Line::new(Rc::clone(&buffer), 19, 21);

        assert_eq!(line1 < line2, true);
        assert_eq!(line1 > line3, true);
        assert_eq!(line1 < line4, true);
        assert_eq!(line2 == line4, true);
    }
}

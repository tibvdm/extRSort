use std::{rc::Rc, io::Write};

#[derive(Clone, PartialEq, Eq)]
pub struct Line {
    pub buffer: Rc<Vec<u8>>,

    pub start: usize,
    pub end: usize,
}

impl Line {
    pub fn new(buffer: Rc<Vec<u8>>, start: usize, end: usize) -> Self {
        Line { buffer, start, end }
    }

    pub fn write(&self, writer: &mut impl Write) {
        writer.write_all(self.as_bytes()).unwrap();
        writer.write("\n".as_bytes()).unwrap();
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer[self.start..=self.end]
    }
}

unsafe impl Send for Line {}

impl PartialOrd for Line {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.as_bytes().partial_cmp(self.as_bytes())
    }   
}

impl Ord for Line {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.as_bytes().cmp(self.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_cmp() {
        let buffer = Rc::new("AAACL\nAAA\nCAAALTER\nAAA\n".as_bytes().to_vec());

        let line1 = Line::new(Rc::clone(&buffer), 0, 4);
        let line2 = Line::new(Rc::clone(&buffer), 6, 8);
        let line3 = Line::new(Rc::clone(&buffer), 10, 17);
        let line4 = Line::new(Rc::clone(&buffer), 19, 21);

        assert_eq!(line1 > line2, true);
        assert_eq!(line1 < line3, true);
        assert_eq!(line1 > line4, true);
        assert_eq!(line2 == line4, true);
    }
}

use std::{rc::Rc, io::Write, str::from_utf8, fmt::Debug};

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

impl Debug for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", from_utf8(self.as_bytes()).unwrap())
    }
}

impl PartialEq for Line {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl PartialOrd for Line {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }   
}

impl Eq for Line {}

impl Ord for Line {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use crate::line;

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

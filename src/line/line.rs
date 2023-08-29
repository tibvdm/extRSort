use std::io::Write;

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
pub struct Line<'a> {
    pub content: &'a str
}

impl<'a> Line<'a> {
    pub fn write(&self, writer: &mut impl Write) {
        writer.write_all(self.content.as_bytes()).unwrap();
        writer.write_all(b"\n").unwrap();
    }
}

impl<'a> From<&'a str> for Line<'a> {
    fn from(content: &'a str) -> Self {
        Line { content }
    }
}

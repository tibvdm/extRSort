use bytesize::MB;

#[derive(Clone)]
pub struct Configuration {
    pub threads: usize,
    pub buffer_size: usize,
    pub chunk_size: usize,
    pub delimiter: u8,
    pub field: usize // Maybe multiple fields in the future
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            threads: 4,
            buffer_size: 400 * MB as usize,
            chunk_size: 16,
            delimiter: b'\t',
            field: 0
        }
    }
}

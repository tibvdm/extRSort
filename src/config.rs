use bytesize::MB;

pub struct Configuration {
    pub threads: usize,
    pub buffer_size: usize
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            threads: 4,
            buffer_size: 400 * MB as usize
        }
    }
}

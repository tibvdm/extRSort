use std::{io::{self, BufReader, BufWriter}, path::PathBuf};

use sorter::{TmpDirBuilder, external_sort, Configuration};
use structopt::StructOpt;

fn main() {
    let args = SortArgs::from_args();

    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut input_reader = BufReader::new(stdin.lock());
    let mut output_writer = BufWriter::new(stdout.lock());

    let tmp_location = PathBuf::from("/tmp");
    let mut tmp_dir = TmpDirBuilder::new().with_location(&tmp_location).build();

    let config = Configuration {
        buffer_size: args.buffer_size,
        threads: args.threads,
        delimiter: args.delimiter,
        field: args.field,
        ..Configuration::default()
    };

    external_sort(
        &mut input_reader,
        &mut output_writer,
        &mut tmp_dir,
        config
    );
}

#[derive(Debug, StructOpt)]
pub struct SortArgs {
    /// Directory to store temporary files
    #[structopt(short = "t", long = "temp-dir", default_value = "/tmp/sort-rs", parse(from_os_str))]
    pub tmp_dir: PathBuf,

    /// Buffer size in bytes
    #[structopt(short = "b", long = "buffer-size", default_value = "400000000")]
    pub buffer_size: usize,

    /// Number of threads to use
    #[structopt(short = "p", long = "parallel", default_value = "4")]
    pub threads: usize,

    /// Delimiter to use
    #[structopt(short = "d", long = "delimiter", default_value = "\t", parse(try_from_str = parse_delimiter))]
    pub delimiter: u8,

    /// Field to sort on
    #[structopt(short = "f", long = "field", default_value = "1")]
    pub field: usize
}

fn parse_delimiter(s: &str) -> Result<u8, String> {
    s.chars().next().ok_or_else(|| "Invalid delimiter".to_string()).map(|c| c as u8)
}

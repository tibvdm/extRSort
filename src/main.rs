use std::{io::{self, BufReader, BufWriter}, path::PathBuf};
use std::fs::File;

use sorter::{TmpDirBuilder, external_sort, Configuration};
use structopt::StructOpt;

fn main() {
    let args = SortArgs::from_args();

    let stdin = io::stdin();
    let stdout = io::stdout();

    //let stdout = File::create("/dev/null").unwrap();

    let mut input_reader = BufReader::new(stdin.lock());
    let mut output_writer = BufWriter::new(stdout.lock());

    let tmp_location = PathBuf::from("/tmp");
    let mut tmp_dir = TmpDirBuilder::new().with_location(&tmp_location).build();

    let config = Configuration {
        buffer_size: args.buffer_size,
        threads: args.threads,
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
    #[structopt(short = "b", long = "buffer-size", default_value = "300000000")]
    pub buffer_size: usize,

    /// Number of threads to use
    #[structopt(short = "p", long = "parallel", default_value = "4")]
    pub threads: usize
}

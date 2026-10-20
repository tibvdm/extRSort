use std::{io::{self, BufReader, BufWriter, Write}, path::PathBuf, process::exit};

use bytesize::ByteSize;
use sorter::{TmpDirBuilder, external_sort, Configuration};
use clap::Parser;

fn main() {
    let args = SortArgs::parse();

    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut input_reader = BufReader::new(stdin.lock());
    let mut output_writer = BufWriter::new(stdout.lock());

    let mut tmp_dir = TmpDirBuilder::new().with_location(&args.tmp_dir).build();

    let config = Configuration {
        buffer_size: args.buffer_size,
        threads: args.threads,
        delimiter: args.delimiter,
        field: args.field,
        ..Configuration::default()
    };

    // The flush is checked here: dropping output_writer would ignore a failed last write
    let sorted = external_sort(
        &mut input_reader,
        &mut output_writer,
        &mut tmp_dir,
        config
    ).and_then(|_| output_writer.flush());

    if let Err(err) = sorted {
        eprintln!("sorter: {}", err);

        // exit skips destructors, so the temporary directory is removed here
        drop(tmp_dir);
        exit(1);
    }
}

#[derive(Debug, Parser)]
pub struct SortArgs {
    /// Directory to store temporary files
    #[arg(short = 't', long = "temp-dir", default_value = "/tmp/sort-rs")]
    pub tmp_dir: PathBuf,

    /// Buffer size: bytes, or a size such as 400MB, 2G (2 * 10^9) or 2GiB (2 * 2^30)
    #[arg(short = 'b', long = "buffer-size", default_value = "400MB", value_parser = parse_size)]
    pub buffer_size: usize,

    /// Number of threads to use
    #[arg(short = 'p', long = "parallel", default_value_t = 4, value_parser = parse_threads)]
    pub threads: usize,

    /// Delimiter to use
    #[arg(short = 'd', long = "delimiter", default_value = "\t", value_parser = parse_delimiter)]
    pub delimiter: u8,

    /// Field to sort on
    #[arg(short = 'f', long = "field", default_value_t = 1)]
    pub field: usize
}

fn parse_delimiter(s: &str) -> Result<u8, String> {
    // The delimiter is compared as one byte, so it has to be one ASCII character
    match s.as_bytes() {
        [byte] if byte.is_ascii() => Ok(*byte),
        _ => Err("the delimiter must be one ASCII character".to_string())
    }
}

fn parse_threads(s: &str) -> Result<usize, String> {
    match s.parse::<usize>() {
        Ok(0) => Err("at least one thread is needed".to_string()),
        Ok(threads) => Ok(threads),
        Err(err) => Err(err.to_string())
    }
}

fn parse_size(s: &str) -> Result<usize, String> {
    s.parse::<ByteSize>().map(|size| size.as_u64() as usize)
}

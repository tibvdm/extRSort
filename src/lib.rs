use std::io::{Read, Write};

use chunk::Chunks;
use crate::tempfile::TmpDir;
use threadpool::ThreadPool;

mod config;
mod chunk;
mod line;
mod tempfile;
mod sort;
mod merge;
mod util;

pub use crate::config::Configuration;
pub use crate::tempfile::TmpDirBuilder;

pub fn external_sort(
    input: &mut impl Read,
    output: &mut impl Write,
    tmp_dir: &mut TmpDir,
    config: Configuration
) {
    // Threadpool for sorting and mergin chunks
    let threadpool = ThreadPool::new(config.threads);

    // Create a chunk iterator over the input stream
    let mut input_chunks = Chunks::new(input, config.buffer_size);

    // Stage 1: Sort all chunks and write them to small temporary files
    let stage1_tmp_files = sort::sort(&mut input_chunks, &threadpool, tmp_dir, &config);

    // Stage 2: Merge all temporary files into multiple bigger temporary files
    let stage2_tmp_files = merge::merge(stage1_tmp_files, &threadpool, tmp_dir, &config);

    // Stage 3: Merge all temporary files into the output stream
    merge::merge_and_write(stage2_tmp_files, output);
}

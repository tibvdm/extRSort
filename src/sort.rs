use std::{io::{self, Read, Write}, panic::{catch_unwind, AssertUnwindSafe}, sync::mpsc::channel};

use threadpool::ThreadPool;

use crate::{chunk::{Chunks, Chunk}, tempfile::{TmpDir, ClosedTmpFile}, Configuration};

pub fn sort(
    input_chunks: &mut Chunks<impl Read>,
    sorter_pool: &ThreadPool,
    tmp_dir: &mut TmpDir,
    config: &Configuration
) -> io::Result<Vec<ClosedTmpFile>> {
    let (file_sender, file_receiver) = channel();

    let mut tmp_files: Vec<ClosedTmpFile> = vec![];

    let mut started = 0;

    // Create new chunks while inside limits
    for _ in 0..config.threads {
        if let Some(unsorted_chunk) = input_chunks.next() {
            let mut tmp_file = tmp_dir.create_new_file();
            let sender = file_sender.clone();

            sorter_pool.execute(move || {
                let sorted = catch_unwind(AssertUnwindSafe(|| sort_and_write(unsorted_chunk, &mut tmp_file)));
                let _ = sender.send(sorted.map_err(|_| io::Error::other("a sort job failed")).and_then(|_| tmp_file.finish()));
            });

            started += 1;
        }
    }

    // Empty input: no file will be sent, and the receiver below would wait forever
    if started == 0 {
        return Ok(tmp_files);
    }

    // Use an option in order to drop the sender inside the loop
    let mut option_sender = Some(file_sender);

    // While there is at least a single sender connected to this receiver
    while let Ok(file) = file_receiver.recv() {
        // A job that panicked or could not write its file, for example on a full disk, sends an error
        tmp_files.push(file?);

        if let Some(sender) = &option_sender {
            match input_chunks.next() {
                Some(unsorted_chunk) => {
                    let mut tmp_file = tmp_dir.create_new_file();
                    let sender = sender.clone();
                    
                    sorter_pool.execute(move || {
                        let sorted = catch_unwind(AssertUnwindSafe(|| sort_and_write(unsorted_chunk, &mut tmp_file)));
                        let _ = sender.send(sorted.map_err(|_| io::Error::other("a sort job failed")).and_then(|_| tmp_file.finish()));
                    });
                },
                None => option_sender = None
            }
        }
    }

    return Ok(tmp_files);
}

/// Sorts a chunk and writes it to a file
pub fn sort_and_write(mut chunk: Chunk, file: &mut impl Write) {
    chunk.sort_unstable();
    chunk.write(file);
}

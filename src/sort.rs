use std::{io::{Read, Write}, sync::mpsc::channel};

use threadpool::ThreadPool;

use crate::{chunk::{Chunks, Chunk}, tempfile::{TmpDir, ClosedTmpFile, TmpFileOpened}, Configuration};

pub fn sort(
    input_chunks: &mut Chunks<impl Read>,
    sorter_pool: &ThreadPool,
    tmp_dir: &mut TmpDir,
    config: &Configuration
) -> Vec<ClosedTmpFile> {
    let (file_sender, file_receiver) = channel();

    let mut tmp_files: Vec<ClosedTmpFile> = vec![];

    // Create new chunks while inside limits
    for _ in 0..config.threads {
        if let Some(unsorted_chunk) = input_chunks.next() {
            let mut tmp_file = tmp_dir.create_new_file();
            let sender = file_sender.clone();

            sorter_pool.execute(move || {
                sort_and_write(unsorted_chunk, &mut tmp_file);
                let _ = sender.send(tmp_file.close());
            });
        }
    }

    // Use an option in order to drop the sender inside the loop
    let mut option_sender = Some(file_sender);

    // While there is at least a single sender connected to this receiver
    while let Ok(file) = file_receiver.recv() {
        tmp_files.push(file);

        if let Some(sender) = &option_sender {
            match input_chunks.next() {
                Some(unsorted_chunk) => {
                    let mut tmp_file = tmp_dir.create_new_file();
                    let sender = sender.clone();
                    
                    sorter_pool.execute(move || {
                        sort_and_write(unsorted_chunk, &mut tmp_file);
                        let _ = sender.send(tmp_file.close());
                    });
                },
                None => option_sender = None
            }
        }
    }

    return tmp_files;
}

pub fn sort_and_write(mut chunk: Chunk, file: &mut impl Write) {
    chunk.sort_unstable();
    chunk.write(file);
}

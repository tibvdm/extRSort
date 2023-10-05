use std::{sync::mpsc::sync_channel, io::Write};
use std::cmp::{min, max};

use bytesize::MB;
use threadpool::ThreadPool;

use crate::heap::WinnerHeap;
use crate::{tempfile::{ClosedTmpFile, TmpDir, TmpFileReader, TmpFileClosed, TmpFileRead, TmpFileOpened}, util::into_chunks, Configuration, line::{Lines, Line}};

pub fn merge(
    files: Vec<ClosedTmpFile>, 
    sorter_pool: &ThreadPool,
    tmp_dir: &mut TmpDir,
    config: &Configuration
) -> Vec<ClosedTmpFile> {
    let (file_sender, file_reciever) = sync_channel(1);

    let mut tmp_files: Vec<ClosedTmpFile> = vec![];

    // If the amount of files is smaller than chunk_size * threads, then we can 
    // use a smaller chunk size to better distribute the merging work
    let chunk_size = if files.len() < config.chunk_size * config.threads {
        min(config.chunk_size, max(2, files.len() / config.threads))
    } else {
        config.chunk_size
    };

    let mut file_batches = into_chunks(files, chunk_size).into_iter();

    for _ in 0..config.threads {
        if let Some(file_batch) = file_batches.next() {
            let sender = file_sender.clone();
            let mut tmp_file = tmp_dir.create_new_file();
            let config = config.clone();

            sorter_pool.execute(move || {
                merge_and_write(file_batch, &mut tmp_file, config);
                let _ = sender.send(tmp_file.close());
            });
        }
    }

    // Use an option in order to drop the sender inside the loop
    let mut option_sender = Some(file_sender);

    // While there is at least a single sender connected to this receiver
    while let Ok(file) = file_reciever.recv() {
        tmp_files.push(file);

        if let Some(sender) = &option_sender {
            match file_batches.next() {
                Some(file_batch) => {
                    let sender = sender.clone();
                    let mut tmp_file = tmp_dir.create_new_file();
                    let config = config.clone();

                    sorter_pool.execute(move || {
                        merge_and_write(file_batch, &mut tmp_file, config);
                        let _ = sender.send(tmp_file.close());
                    });
                },
                None => option_sender = None
            }
        }
    }

    return tmp_files;
}

pub fn merge_and_write(files: Vec<ClosedTmpFile>, file: &mut impl Write, config: Configuration) {
    let buffer_size = min(40 * MB as usize, config.buffer_size / files.len());

    let mut opened_files: Vec<TmpFileReader> = files
        .into_iter()
        .map(|file| file.reopen())
        .collect();

    let mut lines_iterators: Vec<Lines<&mut TmpFileReader>> = opened_files
        .iter_mut()
        .map(|file| Lines::new(file, buffer_size, config.clone()))
        .collect();

    let mut heap: WinnerHeap<(Line, usize)> = WinnerHeap::new(
        lines_iterators
            .iter_mut()
            .enumerate()
            .map(|(i, lines)| (lines.next().unwrap(), i))
            .collect::<Vec<(Line, usize)>>()
    );

    while let Some((line, lines_index)) = heap.pop() {
        line.write(file);

        if let Some(new_line) = lines_iterators[lines_index].next() {
            heap.push((new_line, lines_index));
        }
    }

    // Remove the temporary files that were merged
    for file in opened_files {
       file.close_and_remove();
    }
}

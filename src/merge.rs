use std::{sync::mpsc::sync_channel, io::Write, collections::BinaryHeap};
use std::cmp::min;

use threadpool::ThreadPool;

use crate::{tempfile::{ClosedTmpFile, TmpDir, TmpFileReader, TmpFileClosed, TmpFileRead, TmpFileOpened}, util::into_chunks, Configuration, line::{Lines, Line}};

pub fn merge(
    files: Vec<ClosedTmpFile>, 
    sorter_pool: &ThreadPool,
    tmp_dir: &mut TmpDir,
    config: &Configuration
) -> Vec<ClosedTmpFile> {
    let (file_sender, file_reciever) = sync_channel(1);

    let mut tmp_files: Vec<ClosedTmpFile> = vec![];

    let chunk_size = if files.len() < config.chunk_size {
        config.chunk_size
    } else {
        min(config.chunk_size, files.len() / config.threads)
    };

    let mut file_batches = into_chunks(files, chunk_size).into_iter();

    for _ in 0..config.threads {
        if let Some(file_batch) = file_batches.next() {
            let sender = file_sender.clone();
            let mut tmp_file = tmp_dir.create_new_file();

            sorter_pool.execute(move || {
                merge_and_write(file_batch, &mut tmp_file);
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

                    sorter_pool.execute(move || {
                        merge_and_write(file_batch, &mut tmp_file);
                        let _ = sender.send(tmp_file.close());
                    });
                },
                None => option_sender = None
            }
        }
    }

    return tmp_files;
}

pub fn merge_and_write(files: Vec<ClosedTmpFile>, file: &mut impl Write) {
    let mut opened_files: Vec<TmpFileReader> = files
        .into_iter()
        .map(|file| file.reopen())
        .collect();

    let mut lines_iterators: Vec<Lines<&mut TmpFileReader>> = opened_files
        .iter_mut()
        .map(|file| Lines::new(file, 8_000))
        .collect();

    let mut heap: BinaryHeap<(Line, usize)> = BinaryHeap::from(
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

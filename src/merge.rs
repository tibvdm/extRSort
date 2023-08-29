use std::{sync::mpsc::sync_channel, io::Write, collections::BinaryHeap};

use threadpool::ThreadPool;

use crate::{tempfile::{ClosedTmpFile, TmpDir, TmpFileReader, TmpFileClosed, TmpFileRead, TmpFileOpened}, util::into_chunks, chunk::{Chunks, ChunkLine}, Configuration};

pub fn merge(
    files: Vec<ClosedTmpFile>, 
    sorter_pool: &ThreadPool,
    tmp_dir: &mut TmpDir,
    config: &Configuration
) -> Vec<ClosedTmpFile> {
    let (file_sender, file_reciever) = sync_channel(1);

    let mut tmp_files: Vec<ClosedTmpFile> = vec![];

    let mut file_batches = into_chunks(files, 20).into_iter();

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

    let mut chunk_iterators: Vec<Chunks<&mut TmpFileReader>> = opened_files
        .iter_mut()
        .map(|file| Chunks::new(file, 8_000))
        .collect();

    let current_chunks: Vec<ChunkLine> = chunk_iterators
        .iter_mut()
        .enumerate()
        .map(|(i, ci)| ChunkLine::new(ci.next(), 0, i))
        .collect();

    let mut heap: BinaryHeap<ChunkLine> = BinaryHeap::from(current_chunks);

    while let Some(smallest) = heap.pop() {
        smallest.current_line().unwrap().write(file);

        let smallest_index = smallest.line_index + 1;
        if smallest_index >= smallest.chunk.as_ref().unwrap().len() {
            let new_chunk = chunk_iterators[smallest.iterator_index].next();
            if new_chunk.is_some() {
                heap.push(ChunkLine::new(new_chunk, 0, smallest.iterator_index));
            }
        } else {
            heap.push(ChunkLine::new(smallest.chunk, smallest_index, smallest.iterator_index));
        }
    }

    // Remove the temporary files that were merged
    for file in opened_files {
        file.close_and_remove();
    }
}

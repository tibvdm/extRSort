use std::{sync::mpsc::sync_channel, io::Write, collections::BinaryHeap};
use std::rc::Rc;

use threadpool::ThreadPool;

use crate::{tempfile::{ClosedTmpFile, TmpDir, TmpFileReader, TmpFileClosed, TmpFileRead, TmpFileOpened}, util::into_chunks, chunk::{Chunks, ChunkLine}, Configuration};
use crate::chunk::Chunk;

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

    let mut current_chunks: Vec<Option<Chunk>> = chunk_iterators
        .iter_mut()
        .map(|ci| ci.next())
        .collect();

    let mut line_indices = vec![0; current_chunks.len()];

    let mut heap: BinaryHeap<ChunkLine> = BinaryHeap::new();
    for i in 0..current_chunks.len() {
        heap.push(ChunkLine::new(current_chunks[i].as_ref().unwrap().lines().get(0).unwrap(), i))
    }

    while !heap.is_empty() {
        let smallest = heap.pop().unwrap();

        smallest.line.write(file);

        let smallest_index = line_indices[smallest.iterator_index] + 1;
        if smallest_index >= current_chunks[smallest.iterator_index].as_ref().unwrap().len() {
            current_chunks[smallest.iterator_index] = chunk_iterators[smallest.iterator_index].next();
            line_indices[smallest.iterator_index] = 0;
            if current_chunks[smallest.iterator_index].is_some() {
                heap.push(ChunkLine::new(
                    current_chunks[smallest.iterator_index].as_ref().unwrap().lines().get(0).unwrap(),
                    smallest.iterator_index
                ));
            }
        }

        else {
            line_indices[smallest.iterator_index] = smallest_index;
            heap.push(ChunkLine::new(
                current_chunks[smallest.iterator_index].as_ref().unwrap().lines().get(smallest_index).unwrap(),
                smallest.iterator_index
            ));
        }
    }

    // Remove the temporary files that were merged
    for file in opened_files {
        file.close_and_remove();
    }
}

use std::{sync::mpsc::sync_channel, io::Write, collections::BinaryHeap};

use threadpool::ThreadPool;

use crate::{tempfile::{ClosedTmpFile, TmpDir, TmpFileReader, TmpFileClosed, TmpFileRead, TmpFileOpened}, util::into_chunks, chunk::{Chunks, Chunk}, Configuration};

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

// struct MergeItem {
    
// }

pub fn merge_and_write(files: Vec<ClosedTmpFile>, file: &mut impl Write) {
    let mut opened_files: Vec<TmpFileReader> = files
        .into_iter()
        .map(|file| file.reopen())
        .collect();

    let chunk_iterators: Vec<Chunks<&mut TmpFileReader>> = opened_files
        .iter_mut()
        .map(|file| Chunks::new(file, 8_000))
        .collect();

    let items: Vec<(Chunk, Chunks<&mut TmpFileReader>)> = chunk_iterators
        .into_iter()
        .map(|mut ci| (ci.next().unwrap(), ci))
        .collect::<Vec<_>>();

    let mut heap: BinaryHeap<(Chunk, Chunks<&mut TmpFileReader>)> = BinaryHeap::from(items);

    while let Some((mut chunk, mut chunk_iterator)) = heap.pop() {
        chunk.next_line().write(file);

        if chunk.is_empty() {
            if let Some(new_chunk) = chunk_iterator.next() {
                heap.push((new_chunk, chunk_iterator))
            }
        } else {
            heap.push((chunk, chunk_iterator));
        }
    }

    // Remove the temporary files that were merged
    for file in opened_files {
       file.close_and_remove();
    }
}

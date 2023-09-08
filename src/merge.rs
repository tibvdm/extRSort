use std::{sync::mpsc::sync_channel, io::Write, collections::BinaryHeap, cell::RefCell, rc::Rc, borrow::Cow};

use threadpool::ThreadPool;

use crate::{tempfile::{ClosedTmpFile, TmpDir, TmpFileReader, TmpFileClosed, TmpFileRead, TmpFileOpened}, util::into_chunks, chunk::{Chunks, ChunkLine, Chunk}, Configuration};

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

    // let current_chunks: RefCell<Vec<Option<Chunk>>> = RefCell::new(chunk_iterators
    //     .iter_mut()
    //     .map(|ci| ci.next())
    //     .collect());

    let mut heap: BinaryHeap<(ChunkLine, Chunk)> = BinaryHeap::from(chunk_iterators
        .iter_mut()
        .enumerate()
        .map(|(i, c)| {
            let mut chunk = c.next().unwrap();
            let line = Cow::Borrowed(chunk.next_line());

            (ChunkLine::new(line, i), chunk)
        })
        .collect::<Vec<_>>()
    );

    while let Some((smallest, mut binding)) = heap.pop() {
        smallest.write(file);

        if binding.is_empty() {
            binding = chunk_iterators[smallest.iterator_index].next().unwrap();
        }

        heap.push((ChunkLine::new(Cow::Borrowed(binding.next_line()), smallest.iterator_index), binding));
    }

    // Remove the temporary files that were merged
    for file in opened_files {
        file.close_and_remove();
    }
}

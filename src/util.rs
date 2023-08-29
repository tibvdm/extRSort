use std::cmp::min;

pub fn into_chunks<T>(mut input: Vec<T>, n: usize) -> Vec<Vec<T>> {
    let mut chunks: Vec<Vec<T>> = vec![];

    while input.len() > 0 {
        let mut chunk = vec![];
        for _ in 0..min(n, input.len()) {
            chunk.push(input.pop().unwrap());
        }
        chunks.push(chunk);
    }

    chunks
}

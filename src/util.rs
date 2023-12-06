use std::cmp::min;

/// Split a vector into chunks of size n.
/// 
/// # Arguments
/// 
/// * `input` - The vector to split
/// * `n` - The maximum size of the chunks
/// 
/// # Returns
/// 
/// A vector of vectors of size n
pub fn into_chunks<T>(mut input: Vec<T>, n: usize) -> Vec<Vec<T>> {
    // TODO: Use Result to handle the case where n is zero

    let mut chunks: Vec<Vec<T>> = vec![];

    // Create chunks while there are still elements left
    while input.len() > 0 {
        let mut chunk = vec![];
        for _ in 0..min(n, input.len()) {
            chunk.push(input.pop().unwrap());
        }
        chunks.push(chunk);
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_chunks() {
        let input = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let chunks = into_chunks(input, 3);

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], vec![8, 7, 6]);
        assert_eq!(chunks[1], vec![5, 4, 3]);
        assert_eq!(chunks[2], vec![2, 1]);
    }
}

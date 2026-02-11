//! A vector-like container storing elements in fixed-size chunks.
//!
//! Avoids large contiguous allocations during growth by allocating many small
//! chunks instead of one large block. Used for vreg_types and facts in VCode
//! to mitigate OOM on memory-constrained systems (e.g. ESP32).

use alloc::vec::Vec;
use core::ops::{Index, IndexMut};

/// Chunk size in elements. Tune this to control maximum allocation size per chunk.
const CHUNK_SIZE: usize = 256;

/// A vector-like container that stores elements in fixed-size chunks.
///
/// Provides O(1) index access, push, and len. No contiguous slice access.
/// Each chunk is a separate allocation, limiting the size of any single
/// allocation during growth.
#[derive(Clone, Default)]
pub struct ChunkedVec<T> {
    chunks: Vec<Vec<T>>,
    len: usize,
}

impl<T> ChunkedVec<T> {
    /// Create an empty ChunkedVec.
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            len: 0,
        }
    }

    /// Create a ChunkedVec with capacity for at least `cap` elements.
    /// Pre-allocates chunks so subsequent `resize`/`push` up to `cap` do not allocate.
    pub fn with_capacity(cap: usize) -> Self {
        let n_chunks = (cap + CHUNK_SIZE - 1) / CHUNK_SIZE;
        let mut chunks = Vec::with_capacity(n_chunks.max(1));
        // Pre-allocate first chunk if we need capacity
        if cap > 0 {
            let first_cap = (cap).min(CHUNK_SIZE);
            chunks.push(Vec::with_capacity(first_cap));
        }
        Self { chunks, len: 0 }
    }

    /// Number of elements.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Append an element.
    pub fn push(&mut self, value: T) {
        let offset = self.len % CHUNK_SIZE;

        if offset == 0 && self.len > 0 {
            // Need a new chunk
            self.chunks.push(Vec::with_capacity(CHUNK_SIZE));
        }
        if offset == 0 && self.len == 0 && self.chunks.is_empty() {
            self.chunks.push(Vec::with_capacity(CHUNK_SIZE));
        }

        let last = self.chunks.len() - 1;
        self.chunks[last].push(value);
        self.len += 1;
    }

    /// Resize to `new_len`, filling new slots with `value` if growing.
    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        if new_len <= self.len {
            self.len = new_len;
            // Truncate chunks: remove excess chunks and truncate the last one
            let keep_chunks = (new_len + CHUNK_SIZE - 1) / CHUNK_SIZE;
            if keep_chunks == 0 {
                self.chunks.clear();
                return;
            }
            self.chunks.truncate(keep_chunks);
            let last_offset = new_len % CHUNK_SIZE;
            let last_len = if last_offset == 0 && new_len > 0 {
                CHUNK_SIZE
            } else {
                last_offset
            };
            self.chunks[keep_chunks - 1].truncate(last_len);
            return;
        }

        // Grow
        while self.len < new_len {
            self.push(value.clone());
        }
    }

    /// Get reference to element at index.
    #[inline]
    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx >= self.len {
            return None;
        }
        let chunk_idx = idx / CHUNK_SIZE;
        let offset = idx % CHUNK_SIZE;
        self.chunks.get(chunk_idx).and_then(|c| c.get(offset))
    }

    /// Get mutable reference to element at index.
    #[inline]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx >= self.len {
            return None;
        }
        let chunk_idx = idx / CHUNK_SIZE;
        let offset = idx % CHUNK_SIZE;
        self.chunks
            .get_mut(chunk_idx)
            .and_then(|c| c.get_mut(offset))
    }

    /// Iterate over all elements in order.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.chunks.iter().flat_map(|c| c.iter())
    }
}

impl<T> Index<usize> for ChunkedVec<T> {
    type Output = T;

    #[inline]
    fn index(&self, idx: usize) -> &Self::Output {
        self.get(idx).expect("ChunkedVec index out of bounds")
    }
}

impl<T> IndexMut<usize> for ChunkedVec<T> {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        self.get_mut(idx).expect("ChunkedVec index out of bounds")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem;

    fn chunked_vec_new() -> ChunkedVec<i32> {
        ChunkedVec::new()
    }

    #[test]
    fn push_and_len() {
        let mut v = chunked_vec_new();
        assert_eq!(v.len(), 0);
        v.push(1);
        v.push(2);
        v.push(3);
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], 1);
        assert_eq!(v[1], 2);
        assert_eq!(v[2], 3);
    }

    #[test]
    fn get_and_get_mut() {
        let mut v = chunked_vec_new();
        v.push(10);
        v.push(20);
        assert_eq!(v.get(0), Some(&10));
        assert_eq!(v.get(1), Some(&20));
        assert_eq!(v.get(2), None);

        *v.get_mut(1).unwrap() = 99;
        assert_eq!(v[1], 99);
    }

    #[test]
    fn index_bounds() {
        let mut v = chunked_vec_new();
        v.push(42);
        assert_eq!(v[0], 42);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn index_panic_out_of_bounds() {
        let v: ChunkedVec<i32> = ChunkedVec::new();
        let _ = v[0];
    }

    #[test]
    fn resize_grow() {
        let mut v = chunked_vec_new();
        v.resize(5, -1);
        assert_eq!(v.len(), 5);
        for i in 0..5 {
            assert_eq!(v[i], -1);
        }
    }

    #[test]
    fn resize_shrink() {
        let mut v = chunked_vec_new();
        v.resize(10, 0);
        v.resize(3, 0);
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn iter_yields_sequence() {
        let mut v = chunked_vec_new();
        for i in 0..10 {
            v.push(i as i32);
        }
        let collected: Vec<i32> = v.iter().copied().collect();
        assert_eq!(collected, (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn with_capacity_then_many_pushes() {
        let mut v = ChunkedVec::with_capacity(600);
        for i in 0..600 {
            v.push(i as i32);
        }
        assert_eq!(v.len(), 600);
        assert_eq!(v[0], 0);
        assert_eq!(v[299], 299);
        assert_eq!(v[599], 599);
    }

    #[test]
    fn take_leaves_empty() {
        let mut v = chunked_vec_new();
        v.push(1);
        v.push(2);
        let taken = mem::take(&mut v);
        assert_eq!(taken.len(), 2);
        assert_eq!(v.len(), 0);
    }
}

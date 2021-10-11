//![![ci-tests](https://github.com/arindas/bheap/actions/workflows/ci-tests.yml/badge.svg)](https://github.com/arindas/bheap/actions/workflows/ci-tests.yml)
//![![rustdoc](https://github.com/arindas/bheap/actions/workflows/rustdoc.yml/badge.svg)](https://github.com/arindas/bheap/actions/workflows/rustdoc.yml)
//!
//!A generic binary max heap implementation for implementing a dynamically prioritizable priority queue.
//!
//!This implementation uses a vector as the underlying data-structure. Hence, there is no oppurtunity
//!for fine grained locking. Users of this crate are request to wrap `bheap::BinaryMaxHeap` with the
//!required concurrency primitive for use in multithreaded contexts.
//!
//!## Why is this necessary?
//!The binary heap implementation provided by the standard library (`use std::collections::binary_heap::BinaryHeap;`),
//!assumes that the ordering of the elements in the domain is fixed. I needed a binary heap implementation which allowed
//!for change in ordering of elements at runtime.
//!
//!## How does it work?
//!`bheap::BinaryMaxHeap` enforces the `Ord + bheap::Uid` trait bounds on the element type. The `Uid` trait, simply
//!presents a method for returing a unique `u64` uid for the type.
//!
//!The struct maintains a `Vec<T>` as the underlying storage buffer and a `HashMap<u64, usize>` for maintaining a
//!mapping from `T::uid()` to position in vector. This map is updated on every heap operation to remain consistent.
//!
//!When the ordering of an element changes, its position in the heap can be looked up in the heap using the
//!hashmap. Then, we `heapify_up()` or `heapify_down()` as required to restore heap property.
//!
//!## Limitations
//!Since, we use `u64` for uniquely identitfying elements, this heap can only scale up `2^64 = 18446744073709551616` elements.
//!This was more than enough for my purposes.


use std::{cmp::Ordering, collections::HashMap};

pub trait Uid {
    fn uid(&self) -> u64;
}

pub struct BinaryMaxHeap<T>
where
    T: Ord + Uid,
{
    buffer: Vec<T>,
    index: HashMap<u64, usize>,
}

impl<T> BinaryMaxHeap<T>
where
    T: Ord + Uid,
{
    pub fn from_vec(buffer: Vec<T>) -> Self {
        let mut bheap = BinaryMaxHeap {
            buffer,
            index: HashMap::new(),
        };

        if !bheap.is_empty() {
            bheap.build_heap();
        }

        bheap
    }

    pub fn new() -> Self {
        BinaryMaxHeap::from_vec(vec![])
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    fn swap_elems_at_indices(&mut self, i: usize, j: usize) {
        let index = &mut self.index;

        index.insert(self.buffer[i].uid(), j);
        index.insert(self.buffer[j].uid(), i);

        self.buffer.swap(i, j);
    }

    #[inline]
    fn cmp(&self, i: usize, j: usize) -> Ordering {
        self.buffer[i].cmp(&self.buffer[j])
    }

    fn heapify_up(&mut self, idx: usize) -> Option<usize> {
        let mut i = idx;

        while i > 0 {
            let parent = (i - 1) / 2;

            if let Ordering::Greater = self.cmp(i, parent) {
                self.swap_elems_at_indices(i, parent);
                i = parent;
            } else {
                break;
            };
        }

        if i != idx {
            return Some(i);
        } else {
            return None;
        }
    }

    fn heapify_dn(&mut self, idx: usize) -> Option<usize> {
        let mut i = idx;

        while i < (self.len() / 2) {
            let mut max = i;
            let (lc, rc) = (2 * i + 1, 2 * i + 2);

            if lc < self.len() {
                if let Ordering::Less = self.cmp(max, lc) {
                    max = lc;
                }
            }

            if rc < self.len() {
                if let Ordering::Less = self.cmp(max, rc) {
                    max = rc;
                }
            }

            if i != max {
                self.swap_elems_at_indices(i, max);
                i = max;
            } else {
                break;
            }
        }

        if i != idx {
            return Some(i);
        } else {
            return None;
        }
    }

    #[inline]
    fn update_index(&mut self, i: usize) -> Option<usize> {
        if i >= self.len() {
            return None;
        }

        self.index.insert(self.buffer[i].uid(), i)
    }

    pub fn push(&mut self, elem: T) {
        let idx = self.buffer.len();

        self.buffer.push(elem);
        self.update_index(idx);

        self.heapify_up(idx);
    }

    pub fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }

        Some(&self.buffer[0])
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let elem = self.buffer.swap_remove(0);
        self.index.remove(&elem.uid());

        self.update_index(0).and(self.heapify_dn(0));

        Some(elem)
    }

    pub fn build_index(&mut self) {
        for i in 0..self.len() {
            self.update_index(i);
        }
    }

    pub fn build_heap(&mut self) {
        self.build_index();

        for i in (0..(self.len() / 2)).rev() {
            self.heapify_dn(i);
        }
    }

    pub fn restore_heap_property(&mut self, idx: usize) -> Option<usize> {
        if idx >= self.len() {
            return None;
        }

        self.heapify_up(idx).or(self.heapify_dn(idx))
    }

    pub fn index_in_heap_from_uid(&self, uid: u64) -> Option<usize> {
        self.index.get(&uid).map(|&elem_idx| elem_idx)
    }

    pub fn index_in_heap(&self, elem: &T) -> Option<usize> {
        self.index.get(&elem.uid()).map(|&elem_idx| elem_idx)
    }
}

impl<T> BinaryMaxHeap<T>
where
    T: Ord + Uid,
{
    pub(crate) fn _index_consistent(&self) -> bool {
        let mut result = true;
        let mut i = 0;

        for elem in &self.buffer {
            let elem_consistent = self
                .index_in_heap(elem)
                .map_or(false, |elem_idx| elem_idx == i);

            result = result && elem_consistent;
            i += 1
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::{BinaryMaxHeap, Uid};

    impl Uid for u32 {
        fn uid(&self) -> u64 {
            (*self).into()
        }
    }

    #[test]
    fn empty_binary_max_heap() {
        let mut heap = BinaryMaxHeap::<u32>::new();

        assert_eq!(heap.is_empty(), true);
        assert_eq!(heap.len(), 0);
        assert_eq!(heap.peek(), None);
        assert_eq!(heap.pop(), None);

        let mut heap = BinaryMaxHeap::from_vec(Vec::<u32>::with_capacity(10));

        assert_eq!(heap.is_empty(), true);
        assert_eq!(heap.len(), 0);
        assert_eq!(heap.peek(), None);
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn binary_max_heap_from_vec_with_elems() {
        let mut heap = BinaryMaxHeap::from_vec(vec![1, 7, 2, 5, 10, 9]);
        assert_eq!(heap.peek(), Some(&10));
        assert_eq!(heap.pop(), Some(10));

        assert_eq!(heap.peek(), Some(&9));
        assert_eq!(heap.pop(), Some(9));

        assert_eq!(heap.peek(), Some(&7));
        assert_eq!(heap.pop(), Some(7));

        assert_eq!(heap.peek(), Some(&5));
        assert_eq!(heap.pop(), Some(5));

        assert_eq!(heap.peek(), Some(&2));
        assert_eq!(heap.pop(), Some(2));

        assert_eq!(heap.peek(), Some(&1));
        assert_eq!(heap.pop(), Some(1));

        assert_eq!(heap.peek(), None);
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn push_peek_pop_index_correctness() {
        let mut heap = BinaryMaxHeap::<u32>::new();

        heap.push(1);
        assert_eq!(heap.peek(), Some(&1));
        assert!(heap._index_consistent());

        heap.push(7);
        assert_eq!(heap.peek(), Some(&7));
        assert!(heap._index_consistent());

        heap.push(2);
        assert_eq!(heap.peek(), Some(&7));
        assert!(heap._index_consistent());

        heap.push(5);
        assert_eq!(heap.peek(), Some(&7));
        assert!(heap._index_consistent());

        heap.push(10);
        assert_eq!(heap.peek(), Some(&10));
        assert!(heap._index_consistent());

        heap.push(9);
        assert_eq!(heap.peek(), Some(&10));
        assert!(heap._index_consistent());

        assert_eq!(heap.peek(), Some(&10));
        assert_eq!(heap.pop(), Some(10));
        assert!(heap._index_consistent());

        assert_eq!(heap.peek(), Some(&9));
        assert_eq!(heap.pop(), Some(9));
        assert!(heap._index_consistent());

        assert_eq!(heap.peek(), Some(&7));
        assert_eq!(heap.pop(), Some(7));
        assert!(heap._index_consistent());

        assert_eq!(heap.peek(), Some(&5));
        assert_eq!(heap.pop(), Some(5));
        assert!(heap._index_consistent());

        assert_eq!(heap.peek(), Some(&2));
        assert_eq!(heap.pop(), Some(2));
        assert!(heap._index_consistent());

        assert_eq!(heap.peek(), Some(&1));
        assert_eq!(heap.pop(), Some(1));
        assert!(heap._index_consistent());

        assert_eq!(heap.peek(), None);
        assert_eq!(heap.pop(), None);
        assert!(heap._index_consistent());
    }
}

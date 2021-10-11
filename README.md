# bheap

A generic binary max heap implementation for implementing a dynamically prioritizable priority queue.

This implementation uses a vector as the underlying data-structure. Hence, there is no oppurtunity
for fine grained locking. Users of this crate are request to wrap `bheap::BinaryMaxHeap` with the
required concurrency primitive for use in multithreaded contexts.

## Why is this necessary?
The binary heap implementation provided by the standard library (`use std::collections::binary_heap::BinaryHeap;`),
assumes that the ordering of the elements in the domain is fixed. I needed a binary heap implementation which allowed
for change in ordering of elements at runtime.

## How does it work?
`bheap::BinaryMaxHeap` enforces the `Ord + Uid` trait bounds on the element type. The `Uid` trait, simply
presents a method for returing a unique `u64` uid for the type.

The struct maintains a `Vec<T>` as the underlying storage buffer and a `HashMap<u64, usize>` for maintaining a
mapping from `T::uid()` to position in vector. This map is updated on every heap operation to remain consistent.

When the ordering of an element changes, its position in the heap can be looked up in the heap using the
hashmap. Then, we `heapify_up()` or `heapify_down()` as required to restore heap property.
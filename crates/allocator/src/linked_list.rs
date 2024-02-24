//! Lineked list memory allocation.

use linked_list_allocator::Heap;

use core::alloc::Layout;
use core::ptr::NonNull;

use crate::{AllocError, AllocResult, BaseAllocator, ByteAllocator};

/// A byte-granularity memory allocator based on the [linked_list_allocator].
///
/// [linked_list_allocator]: https://docs.rs/linked_list_allocator/0.10.5/linked_list_allocator/
pub struct LinkedListByteAllocator {
    inner: Heap,
}

impl LinkedListByteAllocator {
    /// Creates a new empty `LinkedListByteAllocator`.
    pub const fn new() -> Self {
        Self {
            inner: Heap::empty(),
        }
    }
}

impl BaseAllocator for LinkedListByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        unsafe { self.inner.init(start as *mut u8, size) };
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        // self.inner.extend(by)
        // unsafe { self.inner.add_to_heap(start, start + size) };
        Ok(())
    }
}

impl ByteAllocator for LinkedListByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        self.inner
            .allocate_first_fit(layout)
            .map_err(|_| AllocError::NoMemory)
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        unsafe { self.inner.deallocate(pos, layout) }
    }

    fn total_bytes(&self) -> usize {
        self.inner.size()
    }

    fn used_bytes(&self) -> usize {
        self.inner.used()
    }

    fn available_bytes(&self) -> usize {
        self.inner.size() - self.inner.used()
    }
}

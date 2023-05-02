//! Buddy memory allocation.
//!
//! TODO: more efficient

use buddy_system_allocator::Heap;
use core::alloc::Layout;

use crate::{AllocError, AllocResult, BaseAllocator, ByteAllocator};

/// A byte-granularity memory allocator based on the [buddy_system_allocator].
///
/// [buddy_system_allocator]: https://docs.rs/buddy_system_allocator/latest/buddy_system_allocator/
pub struct BuddyByteAllocator {
    inner: Heap<32>,
}

impl BuddyByteAllocator {
    /// Creates a new empty `BuddyByteAllocator`.
    pub const fn new() -> Self {
        Self {
            inner: Heap::<32>::new(),
        }
    }
}

impl BaseAllocator for BuddyByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        unsafe { self.inner.init(start, size) };
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        unsafe { self.inner.add_to_heap(start, start + size) };
        Ok(())
    }
}

impl ByteAllocator for BuddyByteAllocator {
    fn alloc(&mut self, size: usize, align_pow2: usize) -> AllocResult<usize> {
        self.inner
            .alloc(Layout::from_size_align(size, align_pow2).unwrap())
            .map(|ptr| ptr.as_ptr() as usize)
            .map_err(|_| AllocError::NoMemory)
    }

    fn dealloc(&mut self, pos: usize, size: usize, align_pow2: usize) {
        self.inner.dealloc(
            unsafe { core::ptr::NonNull::new_unchecked(pos as *mut u8) },
            Layout::from_size_align(size, align_pow2).unwrap(),
        )
    }

    fn total_bytes(&self) -> usize {
        self.inner.stats_total_bytes()
    }

    fn used_bytes(&self) -> usize {
        self.inner.stats_alloc_actual()
    }

    fn available_bytes(&self) -> usize {
        self.inner.stats_total_bytes() - self.inner.stats_alloc_actual()
    }
}

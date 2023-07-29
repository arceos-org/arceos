//! Slab memory allocation.
//!
//! TODO: comments

use super::{AllocError, AllocResult, BaseAllocator, ByteAllocator};
use core::alloc::Layout;
use core::num::NonZeroUsize;
use slab_allocator::Heap;

/// A byte-granularity memory allocator based on the [slab allocator].
///
/// [slab allocator]: ../slab_allocator/index.html
pub struct SlabByteAllocator {
    inner: Option<Heap>,
}

impl SlabByteAllocator {
    /// Creates a new empty `SlabByteAllocator`.
    pub const fn new() -> Self {
        Self { inner: None }
    }

    fn inner_mut(&mut self) -> &mut Heap {
        self.inner.as_mut().unwrap()
    }

    fn inner(&self) -> &Heap {
        self.inner.as_ref().unwrap()
    }
}

impl BaseAllocator for SlabByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.inner = unsafe { Some(Heap::new(start, size)) };
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        unsafe {
            self.inner_mut().add_memory(start, size);
        }
        Ok(())
    }
}

impl ByteAllocator for SlabByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonZeroUsize> {
        self.inner_mut()
            .allocate(layout)
            .map(|addr| NonZeroUsize::new(addr).unwrap())
            .map_err(|_| AllocError::NoMemory)
    }

    fn dealloc(&mut self, pos: NonZeroUsize, layout: Layout) {
        unsafe { self.inner_mut().deallocate(pos.get(), layout) }
    }

    fn total_bytes(&self) -> usize {
        self.inner().total_bytes()
    }

    fn used_bytes(&self) -> usize {
        self.inner().used_bytes()
    }

    fn available_bytes(&self) -> usize {
        self.inner().available_bytes()
    }
}

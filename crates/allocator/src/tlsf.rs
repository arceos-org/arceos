//! TLSF memory allocation.
//!
//!

use super::{AllocError, AllocResult, BaseAllocator, ByteAllocator};
use core::alloc::Layout;
use tlsf_allocator::HeapRust;

/// A byte-granularity memory allocator based on the [tlsf_allocator] written by rust code.
pub struct TLSFAllocator {
    inner: Option<HeapRust>,
}

impl TLSFAllocator {
    /// Creates a new empty `TLSFAllocator`.
    pub const fn new() -> Self {
        Self { inner: None }
    }

    fn inner_mut(&mut self) -> &mut HeapRust {
        self.inner.as_mut().unwrap()
    }

    fn inner(&self) -> &HeapRust {
        self.inner.as_ref().unwrap()
    }
}

impl BaseAllocator for TLSFAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.inner = Some(HeapRust::new());
        self.inner_mut().init(start, size);
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        self.inner_mut().add_memory(start, size);
        Ok(())
    }
}

impl ByteAllocator for TLSFAllocator {
    fn alloc(&mut self, size: usize, align_pow2: usize) -> AllocResult<usize> {
        self.inner_mut()
            .allocate(Layout::from_size_align(size, align_pow2).unwrap())
            .map_err(|_| AllocError::NoMemory)
    }

    fn dealloc(&mut self, pos: usize, size: usize, align_pow2: usize) {
        self.inner_mut()
            .deallocate(pos, Layout::from_size_align(size, align_pow2).unwrap())
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

//! TLSF memory allocation.
//!
//!

use super::{AllocError, AllocResult, BaseAllocator, ByteAllocator};
use tlsf_allocator::HeapC;

pub struct TLSFCAllocator {
    inner: Option<HeapC>,
}

impl TLSFCAllocator {
    pub const fn new() -> Self {
        Self { inner: None }
    }

    fn inner_mut(&mut self) -> &mut HeapC {
        self.inner.as_mut().unwrap()
    }

    fn inner(&self) -> &HeapC {
        self.inner.as_ref().unwrap()
    }
}

impl BaseAllocator for TLSFCAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.inner = Some(HeapC::new());
        self.inner_mut().init(start, size);
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        self.inner_mut().add_memory(start, size);
        Ok(())
    }
}

impl ByteAllocator for TLSFCAllocator {
    fn alloc(&mut self, size: usize, align_pow2: usize) -> AllocResult<usize> {
        self.inner_mut()
            .allocate(size, align_pow2)
            .map_err(|_| AllocError::NoMemory)
    }

    fn dealloc(&mut self, pos: usize, size: usize, align_pow2: usize) {
        self.inner_mut().deallocate(pos, size, align_pow2)
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

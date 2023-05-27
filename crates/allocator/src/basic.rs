//! Basic memory allocation.
//!
//!

use super::{AllocError, AllocResult, BaseAllocator, ByteAllocator};
use basic_allocator::Heap;
use core::alloc::Layout;

pub struct BasicAllocator {
    inner: Option<Heap>,
}

impl BasicAllocator {
    pub const fn new() -> Self {
        Self { inner: None }
    }

    fn inner_mut(&mut self) -> &mut Heap {
        self.inner.as_mut().unwrap()
    }

    fn inner(&self) -> &Heap {
        self.inner.as_ref().unwrap()
    }

    pub fn set_strategy(&mut self, strategy: &str) {
        self.inner_mut().set_strategy(strategy);
    }
}

impl BaseAllocator for BasicAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.inner = Some(Heap::new());
        self.inner_mut().init(start, size);
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        self.inner_mut().add_memory(start, size);
        Ok(())
    }
}

impl ByteAllocator for BasicAllocator {
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

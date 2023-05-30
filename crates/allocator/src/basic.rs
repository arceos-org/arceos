//! Basic memory allocation.
//!
//!

use super::{AllocError, AllocResult, BaseAllocator, ByteAllocator};
use basic_allocator::Heap;
use core::alloc::Layout;

pub struct BasicAllocator<const STRATEGY: usize> {
    inner: Option<Heap>,
}

impl<const S: usize> BasicAllocator<S> {
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

impl<const S: usize> BaseAllocator for BasicAllocator<S> {
    fn init(&mut self, start: usize, size: usize) {
        self.inner = Some(Heap::new());
        match S {
            0 => {
                self.inner_mut().init(start, size, "first_fit");
            }
            1 => {
                self.inner_mut().init(start, size, "best_fit");
            }
            2 => {
                self.inner_mut().init(start, size, "worst_fit");
            }
            _ => {
                panic!("Unknown basic allocator strategy.");
            }
        }
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        self.inner_mut().add_memory(start, size);
        Ok(())
    }
}

impl<const S: usize> ByteAllocator for BasicAllocator<S> {
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

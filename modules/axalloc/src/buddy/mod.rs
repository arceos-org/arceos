use buddy_system_allocator::LockedHeap;
use core::ptr::NonNull;

use super::{AllocatorPtr, AxAllocator, Layout};

pub struct BuddyAllocator {
    inner: LockedHeap<32>,
}

impl BuddyAllocator {
    pub const fn new() -> Self {
        Self {
            inner: LockedHeap::<32>::new(),
        }
    }
}

impl AxAllocator for BuddyAllocator {
    fn add_mem_region(&self, start: AllocatorPtr, size: usize) {
        unsafe { self.inner.lock().add_to_heap(start, start + size) }
    }

    fn alloc(&self, layout: Layout) -> Result<AllocatorPtr, ()> {
        self.inner
            .lock()
            .alloc(layout)
            .map(|ptr| ptr.as_ptr() as AllocatorPtr)
    }

    fn dealloc(&self, ptr: AllocatorPtr, layout: Layout) {
        self.inner
            .lock()
            .dealloc(unsafe { NonNull::new_unchecked(ptr as *mut u8) }, layout)
    }

    fn used_bytes(&self) -> usize {
        self.inner.lock().stats_alloc_actual()
    }

    fn available_bytes(&self) -> usize {
        let inner = self.inner.lock();
        inner.stats_total_bytes() - inner.stats_alloc_actual()
    }
}

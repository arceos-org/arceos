#![no_std]
#![feature(alloc_error_handler)]

cfg_if::cfg_if! {
    if #[cfg(feature = "buddy")] {
        mod buddy;
        pub use buddy::BuddyAllocator;
        use BuddyAllocator as DefaultAllocator;
    }
}

#[macro_use]
extern crate log;
extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};

type AllocatorPtr = usize;

pub trait AxAllocator {
    /// Add a free memory region to the allocator.
    fn add_mem_region(&self, start: AllocatorPtr, size: usize);

    /// Allocate memory with the given `layout`, returns the memory pointer.
    fn alloc(&self, layout: Layout) -> Result<AllocatorPtr, ()>;

    /// Deallocate memory at the given `ptr` pointer with the given `layout`.
    fn dealloc(&self, ptr: AllocatorPtr, layout: Layout);

    /// Returns allocated memory size in bytes.
    fn used_bytes(&self) -> usize;

    /// Returns available memory size in bytes.
    fn available_bytes(&self) -> usize;
}

unsafe impl GlobalAlloc for DefaultAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Ok(ptr) = AxAllocator::alloc(self, layout) {
            ptr as _
        } else {
            alloc::alloc::handle_alloc_error(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        AxAllocator::dealloc(self, ptr as _, layout)
    }
}

#[cfg_attr(not(test), global_allocator)]
static GLOBAL_ALLOCATOR: DefaultAllocator = DefaultAllocator::new();

#[cfg(not(test))]
#[alloc_error_handler]
fn handle_alloc_error(layout: Layout) -> ! {
    panic!(
        "Heap allocation error: available_bytes = {}, request = {:?}",
        global_allocator().available_bytes(),
        layout
    );
}

pub fn global_allocator() -> &'static impl AxAllocator {
    &GLOBAL_ALLOCATOR
}

pub fn init(start_vaddr: usize, size: usize) {
    info!(
        "Initializing global heap allocator at: [{:#x}, {:#x})",
        start_vaddr,
        start_vaddr + size
    );
    GLOBAL_ALLOCATOR.add_mem_region(start_vaddr, size);
}

pub fn add_mem_region(start_vaddr: usize, size: usize) {
    info!(
        "Add a memory region to global heap allocator: [{:#x}, {:#x})",
        start_vaddr,
        start_vaddr + size
    );
    GLOBAL_ALLOCATOR.add_mem_region(start_vaddr, size);
}

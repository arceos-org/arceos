#![no_std]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
extern crate alloc;

mod page;

use allocator::{AllocResult, BaseAllocator, ByteAllocator, PageAllocator};
use allocator::{BitmapPageAllocator, SlabByteAllocator};
use core::alloc::{GlobalAlloc, Layout};
use spinlock::SpinNoIrq;

const PAGE_SIZE: usize = 0x1000;
const MIN_HEAP_SIZE: usize = 0x8000; // 32 K

pub use page::GlobalPage;

pub struct GlobalAllocator {
    balloc: SpinNoIrq<SlabByteAllocator>,
    palloc: SpinNoIrq<BitmapPageAllocator<PAGE_SIZE>>,
}

impl GlobalAllocator {
    pub const fn new() -> Self {
        Self {
            balloc: SpinNoIrq::new(SlabByteAllocator::new()),
            palloc: SpinNoIrq::new(BitmapPageAllocator::new()),
        }
    }

    pub fn init(&self, start_vaddr: usize, size: usize) {
        assert!(size > MIN_HEAP_SIZE);
        let init_heap_size = MIN_HEAP_SIZE;
        self.palloc.lock().init(start_vaddr, size);
        let heap_ptr = self
            .alloc_pages(init_heap_size / PAGE_SIZE, PAGE_SIZE)
            .unwrap();
        self.balloc.lock().init(heap_ptr, init_heap_size);
    }

    pub fn add_memory(&self, start_vaddr: usize, size: usize) -> AllocResult {
        self.balloc.lock().add_memory(start_vaddr, size)
    }

    pub fn alloc(&self, size: usize, align_pow2: usize) -> AllocResult<usize> {
        // simple two-level allocator: if no heap memory, allocate from the page allocator.
        let mut balloc = self.balloc.lock();
        loop {
            if let Ok(ptr) = balloc.alloc(size, align_pow2) {
                return Ok(ptr);
            } else {
                let old_size = balloc.total_bytes();
                let expand_size = old_size.max(size).next_power_of_two().max(PAGE_SIZE);
                let heap_ptr = self.alloc_pages(expand_size / PAGE_SIZE, PAGE_SIZE)?;
                debug!(
                    "expand heap memory: [{:#x}, {:#x})",
                    heap_ptr,
                    heap_ptr + expand_size
                );
                balloc.add_memory(heap_ptr, expand_size)?;
            }
        }
    }

    pub fn dealloc(&self, pos: usize, size: usize, align_pow2: usize) {
        self.balloc.lock().dealloc(pos, size, align_pow2)
    }

    pub fn alloc_pages(&self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        self.palloc.lock().alloc_pages(num_pages, align_pow2)
    }

    pub fn dealloc_pages(&self, pos: usize, num_pages: usize) {
        self.palloc.lock().dealloc_pages(pos, num_pages)
    }

    pub fn used_bytes(&self) -> usize {
        self.balloc.lock().used_bytes()
    }

    pub fn available_bytes(&self) -> usize {
        self.balloc.lock().available_bytes()
    }

    pub fn used_pages(&self) -> usize {
        self.palloc.lock().used_pages()
    }

    pub fn available_pages(&self) -> usize {
        self.palloc.lock().available_pages()
    }
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Ok(ptr) = GlobalAllocator::alloc(self, layout.size(), layout.align()) {
            ptr as _
        } else {
            alloc::alloc::handle_alloc_error(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        GlobalAllocator::dealloc(self, ptr as _, layout.size(), layout.align())
    }
}

#[cfg_attr(all(target_os = "none", not(test)), global_allocator)]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::new();

#[cfg(all(target_os = "none", not(test)))]
#[alloc_error_handler]
fn handle_alloc_error(layout: Layout) -> ! {
    panic!(
        "Heap allocation error: available_bytes = {}, request = {:?}",
        global_allocator().available_bytes(),
        layout
    );
}

pub fn global_allocator() -> &'static GlobalAllocator {
    &GLOBAL_ALLOCATOR
}

pub fn global_init(start_vaddr: usize, size: usize) {
    debug!(
        "initialize global allocator at: [{:#x}, {:#x})",
        start_vaddr,
        start_vaddr + size
    );
    GLOBAL_ALLOCATOR.init(start_vaddr, size);
}

pub fn global_add_memory(start_vaddr: usize, size: usize) -> AllocResult {
    debug!(
        "add a memory region to global allocator: [{:#x}, {:#x})",
        start_vaddr,
        start_vaddr + size
    );
    GLOBAL_ALLOCATOR.add_memory(start_vaddr, size)
}

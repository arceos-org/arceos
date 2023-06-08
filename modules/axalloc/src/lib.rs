//! [ArceOS](https://github.com/rcore-os/arceos) global memory allocator.
//!
//! It provides [`GlobalAllocator`], which implements the trait
//! [`core::alloc::GlobalAlloc`]. A static global variable of type
//! [`GlobalAllocator`] is defined with the `#[global_allocator]` attribute, to
//! be registered as the standard library’s default allocator.

#![no_std]

#[macro_use]
extern crate log;
extern crate alloc;

mod page;

use allocator::*;
use core::alloc::{GlobalAlloc, Layout};
use core::mem::size_of;
use spinlock::SpinNoIrq;

const PAGE_SIZE: usize = 0x1000;
cfg_if::cfg_if! {
    if #[cfg(feature = "alloc-mimalloc")]{
        const MIN_HEAP_SIZE: usize = 0x400000; // 4 M
    } else{
        const MIN_HEAP_SIZE: usize = 0x8000; // 32 K
    }
}

pub use page::GlobalPage;

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc-buddy")] {
        pub(crate) type Allocator = allocator::BuddyByteAllocator;
    } else if #[cfg(feature = "alloc-slab")] {
        pub(crate) type Allocator = allocator::SlabByteAllocator;
    } else if #[cfg(feature = "alloc-basic-first_fit")] {
        pub(crate) type Allocator = allocator::BasicAllocator<0>;
    } else if #[cfg(feature = "alloc-basic-best_fit")] {
        pub(crate) type Allocator = allocator::BasicAllocator<1>;
    } else if #[cfg(feature = "alloc-basic-worst_fit")] {
        pub(crate) type Allocator = allocator::BasicAllocator<2>;
    } else if #[cfg(feature = "alloc-tlsf-rust")] {
        pub(crate) type Allocator = allocator::TLSFAllocator;
    } else if #[cfg(feature = "alloc-tlsf-c")] {
        pub(crate) type Allocator = allocator::TLSFCAllocator;
    } else if #[cfg(feature = "alloc-mimalloc")] {
        pub(crate) type Allocator = allocator::MiAllocator;
    }
}

/// The global allocator used by ArceOS.
///
/// It combines a [`ByteAllocator`] and a [`PageAllocator`] into a simple
/// two-level allocator: firstly tries allocate from the byte allocator, if
/// there is no memory, asks the page allocator for more memory and adds it to
/// the byte allocator.
///
/// Currently, [`SlabByteAllocator`] is used as the byte allocator, while
/// [`BitmapPageAllocator`] is used as the page allocator.
pub struct GlobalAllocator {
    balloc: SpinNoIrq<Allocator>,
    palloc: SpinNoIrq<BitmapPageAllocator<PAGE_SIZE>>,
}

impl GlobalAllocator {
    /// Creates an empty [`GlobalAllocator`].
    pub const fn new() -> Self {
        Self {
            balloc: SpinNoIrq::new(Allocator::new()),
            palloc: SpinNoIrq::new(BitmapPageAllocator::new()),
        }
    }

    /// Initializes the allocator with the given region.
    ///
    /// It firstly adds the whole region to the page allocator, then allocates
    /// a small region (32 KB) to initialize the byte allocator. Therefore,
    /// the given region must be larger than 32 KB.
    pub fn init(&self, start_vaddr: usize, size: usize) {
        assert!(size > MIN_HEAP_SIZE);
        let init_heap_size = MIN_HEAP_SIZE;
        self.palloc.lock().init(start_vaddr, size);
        cfg_if::cfg_if! {
            if #[cfg(feature = "alloc-mimalloc")]{
                // mimalloc中，申请的内存必须是4MB对齐的
                let heap_ptr = self
                    .alloc_pages(init_heap_size / PAGE_SIZE, MIN_HEAP_SIZE)
                    .unwrap();
                let new_heap_ptr = heap_ptr;
            } else{
                let heap_ptr = self
                    .alloc_pages(init_heap_size / PAGE_SIZE, PAGE_SIZE)
                    .unwrap();
                let new_heap_ptr = heap_ptr;
            }
        }
        self.balloc.lock().init(new_heap_ptr, init_heap_size);
    }

    /// Add the given region to the allocator.
    ///
    /// It will add the whole region to the byte allocator.
    pub fn add_memory(&self, start_vaddr: usize, size: usize) -> AllocResult {
        self.balloc.lock().add_memory(start_vaddr, size)
    }

    /// Allocate arbitrary number of bytes. Returns the left bound of the
    /// allocated region.
    ///
    /// It firstly tries to allocate from the byte allocator. If there is no
    /// memory, it asks the page allocator for more memory and adds it to the
    /// byte allocator.
    ///
    /// `align_pow2` must be a power of 2, and the returned region bound will be
    ///  aligned to it.
    pub fn alloc(&self, size: usize, align_pow2: usize) -> AllocResult<usize> {
        //默认alloc请求都是8对齐，现在TLSF已经可以支持其他字节的对齐
        // simple two-level allocator: if no heap memory, allocate from the page allocator.
        let mut balloc = self.balloc.lock();
        loop {
            if let Ok(ptr) = balloc.alloc(size, align_pow2) {
                return Ok(ptr);
            } else {
                //申请时要比原始size大一点
                cfg_if::cfg_if! {
                    if #[cfg(feature = "alloc-mimalloc")]{
                        // mimalloc中，申请的内存必须是4MB对齐的，而且要是size的至少8/7倍
                        let expand_size = (size * 8 / 7 + align_pow2 + 6 * size_of::<usize>())
                            .next_power_of_two()
                            .max(MIN_HEAP_SIZE);
                        let heap_ptr = self.alloc_pages(expand_size / PAGE_SIZE, MIN_HEAP_SIZE)?;
                        let new_heap_ptr = heap_ptr;
                    } else{
                        let expand_size = (size + align_pow2 + 6 * size_of::<usize>())
                            .next_power_of_two()
                            .max(PAGE_SIZE);
                        let heap_ptr = self.alloc_pages(expand_size / PAGE_SIZE, PAGE_SIZE)?;
                        let new_heap_ptr = heap_ptr;
                    }
                }
                balloc.add_memory(new_heap_ptr, expand_size)?;
            }
        }
    }

    /// Gives back the allocated region to the byte allocator.
    ///
    /// The region should be allocated by [`alloc`], and `align_pow2` should be
    /// the same as the one used in [`alloc`]. Otherwise, the behavior is
    /// undefined.
    ///
    /// [`alloc`]: GlobalAllocator::alloc
    pub fn dealloc(&self, pos: usize, size: usize, align_pow2: usize) {
        self.balloc.lock().dealloc(pos, size, align_pow2);
    }

    /// Allocates contiguous pages.
    ///
    /// It allocates `num_pages` pages from the page allocator.
    ///
    /// `align_pow2` must be a power of 2, and the returned region bound will be
    /// aligned to it.
    pub fn alloc_pages(&self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        self.palloc.lock().alloc_pages(num_pages, align_pow2)
    }

    /// Gives back the allocated pages starts from `pos` to the page allocator.
    ///
    /// The pages should be allocated by [`alloc_pages`], and `align_pow2`
    /// should be the same as the one used in [`alloc_pages`]. Otherwise, the
    /// behavior is undefined.
    ///
    /// [`alloc_pages`]: GlobalAllocator::alloc_pages
    pub fn dealloc_pages(&self, pos: usize, num_pages: usize) {
        self.palloc.lock().dealloc_pages(pos, num_pages)
    }

    /// Returns the number of total bytes in the byte allocator.
    pub fn total_bytes(&self) -> usize {
        self.balloc.lock().total_bytes()
    }

    /// Returns the number of allocated bytes in the byte allocator.
    pub fn used_bytes(&self) -> usize {
        self.balloc.lock().used_bytes()
    }

    /// Returns the number of available bytes in the byte allocator.
    pub fn available_bytes(&self) -> usize {
        self.balloc.lock().available_bytes()
    }

    /// Returns the number of allocated pages in the page allocator.
    pub fn used_pages(&self) -> usize {
        self.palloc.lock().used_pages()
    }

    /// Returns the number of available pages in the page allocator.
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
pub static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::new();

/// Returns the reference to the global allocator.
pub fn global_allocator() -> &'static GlobalAllocator {
    &GLOBAL_ALLOCATOR
}

/// Initializes the global allocator with the given memory region.
///
/// Note that the memory region bounds are just numbers, and the allocator
/// does not actually access the region. Users should ensure that the region
/// is valid and not being used by others, so that the allocated memory is also
/// valid.
///
/// This function should be called only once, and before any allocation.
pub fn global_init(start_vaddr: usize, size: usize) {
    debug!(
        "initialize global allocator at: [{:#x}, {:#x})",
        start_vaddr,
        start_vaddr + size
    );
    GLOBAL_ALLOCATOR.init(start_vaddr, size);
}

/// Add the given memory region to the global allocator.
///
/// Users should ensure that the region is valid and not being used by others,
/// so that the allocated memory is also valid.
///
/// It's similar to [`global_init`], but can be called multiple times.
pub fn global_add_memory(start_vaddr: usize, size: usize) -> AllocResult {
    debug!(
        "add a memory region to global allocator: [{:#x}, {:#x})",
        start_vaddr,
        start_vaddr + size
    );
    GLOBAL_ALLOCATOR.add_memory(start_vaddr, size)
}

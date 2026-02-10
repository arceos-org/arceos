//! Axvisor-specific memory allocator implementation using buddy-slab-allocator crate.
//!
//! This implementation is designed for virtualization scenarios and provides
//! address translation support.

use super::{UsageKind, Usages};
use buddy_slab_allocator::{AllocResult, PageAllocator};
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};
use kspin::SpinNoIrq;

/// The global allocator instance for axvisor mode.
#[cfg_attr(all(target_os = "none", not(test)), global_allocator)]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::new();

pub use buddy_slab_allocator::AddrTranslator;

const PAGE_SIZE: usize = 0x1000;

/// The global allocator used by ArceOS for axvisor.
///
/// This is an adapter around the buddy_slab_allocator::GlobalAllocator that provides
/// compatibility with the original axalloc API and address translation support.
pub struct GlobalAllocator {
    inner: SpinNoIrq<buddy_slab_allocator::GlobalAllocator<PAGE_SIZE>>,
    usages: SpinNoIrq<Usages>,
}

impl Default for GlobalAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalAllocator {
    /// Creates an empty [`GlobalAllocator`].
    pub const fn new() -> Self {
        Self {
            inner: SpinNoIrq::new(buddy_slab_allocator::GlobalAllocator::<PAGE_SIZE>::new()),
            usages: SpinNoIrq::new(Usages::new()),
        }
    }

    /// Returns the name of the allocator.
    pub const fn name(&self) -> &'static str {
        "buddy-slab-allocator"
    }

    /// Initializes the allocator with the given region.
    pub fn init(&self, start_vaddr: usize, size: usize) {
        info!(
            "Initialize global memory allocator, start_vaddr: {:#x}, size: {:#x}",
            start_vaddr, size
        );
        if let Err(e) = self.inner.lock().init(start_vaddr, size) {
            panic!("Failed to initialize allocator: {:?}", e);
        }
    }

    /// Set the address translator for the allocator.
    pub fn set_addr_translator(&self, translator: &'static dyn AddrTranslator) {
        self.inner.lock().set_addr_translator(translator);
    }

    /// Add the given region to the allocator.
    pub fn add_memory(&self, start_vaddr: usize, size: usize) -> AllocResult {
        info!(
            "Add memory region, start_vaddr: {:#x}, size: {:#x}",
            start_vaddr, size
        );
        self.inner.lock().add_memory(start_vaddr, size)
    }

    /// Allocate arbitrary number of bytes. Returns the left bound of the
    /// allocated region.
    pub fn alloc(&self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let result = self.inner.lock().alloc(layout);
        if let Ok(_ptr) = result {
            self.usages.lock().alloc(UsageKind::RustHeap, layout.size());
        }
        result
    }

    /// Gives back the allocated region to the byte allocator.
    pub fn dealloc(&self, pos: NonNull<u8>, layout: Layout) {
        self.usages
            .lock()
            .dealloc(UsageKind::RustHeap, layout.size());
        self.inner.lock().dealloc(pos, layout);
    }

    /// Allocates contiguous pages.
    pub fn alloc_pages(
        &self,
        num_pages: usize,
        alignment: usize,
        kind: UsageKind,
    ) -> AllocResult<usize> {
        let result = self.inner.lock().alloc_pages(num_pages, alignment);
        if let Ok(_addr) = result {
            let size = num_pages * PAGE_SIZE;
            self.usages.lock().alloc(kind, size);
        }
        result
    }

    /// Allocates contiguous low-memory pages (physical address < 4GiB).
    pub fn alloc_dma32_pages(
        &self,
        num_pages: usize,
        alignment: usize,
        kind: UsageKind,
    ) -> AllocResult<usize> {
        let result = self.inner.lock().alloc_dma32_pages(num_pages, alignment);
        if let Ok(_addr) = result {
            let size = num_pages * PAGE_SIZE;
            self.usages.lock().alloc(kind, size);
        }
        result
    }

    /// Allocates contiguous pages starting from the given address.
    pub fn alloc_pages_at(
        &self,
        start: usize,
        num_pages: usize,
        alignment: usize,
        kind: UsageKind,
    ) -> AllocResult<usize> {
        let result = self
            .inner
            .lock()
            .alloc_pages_at(start, num_pages, alignment);
        if let Ok(_addr) = result {
            let size = num_pages * PAGE_SIZE;
            self.usages.lock().alloc(kind, size);
        }
        result
    }

    /// Gives back the allocated pages starts from `pos` to the page allocator.
    pub fn dealloc_pages(&self, pos: usize, num_pages: usize, kind: UsageKind) {
        let size = num_pages * PAGE_SIZE;
        self.usages.lock().dealloc(kind, size);
        self.inner.lock().dealloc_pages(pos, num_pages);
    }

    /// Returns the number of allocated bytes in the byte allocator.
    #[cfg(feature = "tracking")]
    pub fn used_bytes(&self) -> usize {
        // The buddy-slab-allocator doesn't provide a way to get stats
        0
    }

    /// Returns the number of available bytes in the byte allocator.
    #[cfg(feature = "tracking")]
    pub fn available_bytes(&self) -> usize {
        // The buddy-slab-allocator doesn't provide a way to get stats
        0
    }

    /// Returns the number of allocated pages in the page allocator.
    #[cfg(feature = "tracking")]
    pub fn used_pages(&self) -> usize {
        // The buddy-slab-allocator doesn't provide a way to get stats
        0
    }

    /// Returns the number of available pages in the page allocator.
    #[cfg(feature = "tracking")]
    pub fn available_pages(&self) -> usize {
        // The buddy-slab-allocator doesn't provide a way to get stats
        0
    }

    /// Returns the number of allocated bytes in the byte allocator.
    #[cfg(not(feature = "tracking"))]
    pub fn used_bytes(&self) -> usize {
        0
    }

    /// Returns the number of available bytes in the byte allocator.
    #[cfg(not(feature = "tracking"))]
    pub fn available_bytes(&self) -> usize {
        0
    }

    /// Returns the number of allocated pages in the page allocator.
    #[cfg(not(feature = "tracking"))]
    pub fn used_pages(&self) -> usize {
        0
    }

    /// Returns the number of available pages in the page allocator.
    #[cfg(not(feature = "tracking"))]
    pub fn available_pages(&self) -> usize {
        0
    }

    /// Returns the usage statistics of the allocator.
    pub fn usages(&self) -> Usages {
        *self.usages.lock()
    }
}

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
///
/// # Arguments
///
/// - `start_vaddr`: The starting virtual address of the memory region.
/// - `size`: The size of the memory region in bytes.
/// - `translator`: The address translator for virtualization.
pub fn global_init(start_vaddr: usize, size: usize, translator: &'static dyn AddrTranslator) {
    debug!(
        "initialize global allocator at: [{:#x}, {:#x})",
        start_vaddr,
        start_vaddr + size
    );
    GLOBAL_ALLOCATOR.init(start_vaddr, size);
    GLOBAL_ALLOCATOR.set_addr_translator(translator);
    info!("global allocator initialized");
}

/// Add the given memory region to the global allocator.
///
/// Users should ensure that the region is valid and not being used by others,
/// so that the allocated memory is also valid.
///
/// It's similar to [`global_init`], but can be called multiple times.
///
/// # Arguments
///
/// - `start_vaddr`: The starting virtual address of the memory region.
/// - `size`: The size of the memory region in bytes.
pub fn global_add_memory(start_vaddr: usize, size: usize) -> AllocResult {
    debug!(
        "add a memory region to global allocator: [{:#x}, {:#x})",
        start_vaddr,
        start_vaddr + size
    );
    GLOBAL_ALLOCATOR.add_memory(start_vaddr, size)
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let inner = move || {
            if let Ok(ptr) = GlobalAllocator::alloc(self, layout) {
                ptr.as_ptr()
            } else {
                alloc::alloc::handle_alloc_error(layout)
            }
        };

        #[cfg(feature = "tracking")]
        {
            crate::tracking::with_state(|state| match state {
                None => inner(),
                Some(state) => {
                    let ptr = inner();
                    let generation = state.generation;
                    state.generation += 1;
                    state.map.insert(
                        ptr as usize,
                        crate::tracking::AllocationInfo {
                            layout,
                            backtrace: axbacktrace::Backtrace::capture(),
                            generation,
                        },
                    );
                    ptr
                }
            })
        }

        #[cfg(not(feature = "tracking"))]
        inner()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let ptr = NonNull::new(ptr).expect("dealloc null ptr");
        let inner = || GlobalAllocator::dealloc(self, ptr, layout);

        #[cfg(feature = "tracking")]
        crate::tracking::with_state(|state| match state {
            None => inner(),
            Some(state) => {
                let address = ptr.as_ptr() as usize;
                state.map.remove(&address);
                inner()
            }
        });

        #[cfg(not(feature = "tracking"))]
        inner();
    }
}

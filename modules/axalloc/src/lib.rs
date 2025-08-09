//! [ArceOS](https://github.com/arceos-org/arceos) global memory allocator.
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

use core::{
    alloc::{GlobalAlloc, Layout},
    fmt,
    ptr::NonNull,
};

use allocator::{AllocResult, BaseAllocator, BitmapPageAllocator, ByteAllocator, PageAllocator};
use kspin::SpinNoIrq;

const PAGE_SIZE: usize = 0x1000;

const MIN_HEAP_SIZE: usize = 0x8000; // 32 K

pub use page::GlobalPage;

cfg_if::cfg_if! {
    if #[cfg(feature = "slab")] {
        /// The default byte allocator.
        pub type DefaultByteAllocator = allocator::SlabByteAllocator;
    } else if #[cfg(feature = "buddy")] {
        /// The default byte allocator.
        pub type DefaultByteAllocator = allocator::BuddyByteAllocator;
    } else if #[cfg(feature = "tlsf")] {
        /// The default byte allocator.
        pub type DefaultByteAllocator = allocator::TlsfByteAllocator;
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UsageKind {
    RustHeap,
    UserMem,
    PageCache,
    PageTable,
    Dma,
    Global,
}

#[derive(Clone, Copy)]
pub struct UsageStats([usize; 6]);

impl UsageStats {
    const fn new() -> Self {
        Self([0; 6])
    }

    fn alloc(&mut self, kind: UsageKind, size: usize) {
        self.0[kind as usize] += size;
    }

    fn dealloc(&mut self, kind: UsageKind, size: usize) {
        self.0[kind as usize] -= size;
    }
}

impl fmt::Debug for UsageStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("UsageStats");
        for kind in [
            UsageKind::RustHeap,
            UsageKind::UserMem,
            UsageKind::PageCache,
            UsageKind::PageTable,
            UsageKind::Dma,
            UsageKind::Global,
        ] {
            d.field(
                match kind {
                    UsageKind::RustHeap => "Rust Heap",
                    UsageKind::UserMem => "User Memory",
                    UsageKind::PageCache => "Page Cache",
                    UsageKind::PageTable => "Page Table",
                    UsageKind::Dma => "Dma",
                    UsageKind::Global => "Global",
                },
                &self.0[kind as usize],
            );
        }
        d.finish()
    }
}

/// The global allocator used by ArceOS.
///
/// It combines a [`ByteAllocator`] and a [`PageAllocator`] into a simple
/// two-level allocator: firstly tries allocate from the byte allocator, if
/// there is no memory, asks the page allocator for more memory and adds it to
/// the byte allocator.
///
/// Currently, [`TlsfByteAllocator`] is used as the byte allocator, while
/// [`BitmapPageAllocator`] is used as the page allocator.
///
/// [`TlsfByteAllocator`]: allocator::TlsfByteAllocator
pub struct GlobalAllocator {
    balloc: SpinNoIrq<DefaultByteAllocator>,
    palloc: SpinNoIrq<BitmapPageAllocator<PAGE_SIZE>>,
    stats: SpinNoIrq<UsageStats>,
}

impl GlobalAllocator {
    /// Creates an empty [`GlobalAllocator`].
    pub const fn new() -> Self {
        Self {
            balloc: SpinNoIrq::new(DefaultByteAllocator::new()),
            palloc: SpinNoIrq::new(BitmapPageAllocator::new()),
            stats: SpinNoIrq::new(UsageStats::new()),
        }
    }

    /// Returns the name of the allocator.
    pub const fn name(&self) -> &'static str {
        cfg_if::cfg_if! {
            if #[cfg(feature = "slab")] {
                "slab"
            } else if #[cfg(feature = "buddy")] {
                "buddy"
            } else if #[cfg(feature = "tlsf")] {
                "TLSF"
            }
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
        let heap_ptr = self
            .alloc_pages(init_heap_size / PAGE_SIZE, PAGE_SIZE, UsageKind::RustHeap)
            .unwrap();
        self.balloc.lock().init(heap_ptr, init_heap_size);
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
    fn alloc(&self, layout: Layout) -> AllocResult<NonNull<u8>> {
        // simple two-level allocator: if no heap memory, allocate from the page
        // allocator.
        let mut balloc = self.balloc.lock();
        loop {
            if let Ok(ptr) = balloc.alloc(layout) {
                self.stats.lock().alloc(UsageKind::RustHeap, layout.size());
                return Ok(ptr);
            } else {
                let old_size = balloc.total_bytes();
                let expand_size = old_size
                    .max(layout.size())
                    .next_power_of_two()
                    .max(PAGE_SIZE);

                let mut try_size = expand_size;
                let min_size = PAGE_SIZE.max(layout.size());
                loop {
                    let heap_ptr = match self.alloc_pages(
                        try_size / PAGE_SIZE,
                        PAGE_SIZE,
                        UsageKind::RustHeap,
                    ) {
                        Ok(ptr) => ptr,
                        Err(err) => {
                            try_size /= 2;
                            if try_size < min_size {
                                return Err(err);
                            }
                            continue;
                        }
                    };
                    debug!(
                        "expand heap memory: [{:#x}, {:#x})",
                        heap_ptr,
                        heap_ptr + try_size
                    );
                    balloc.add_memory(heap_ptr, try_size)?;
                    break;
                }
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
    pub fn dealloc(&self, pos: NonNull<u8>, layout: Layout) {
        self.stats
            .lock()
            .dealloc(UsageKind::RustHeap, layout.size());
        self.balloc.lock().dealloc(pos, layout)
    }

    /// Allocates contiguous pages.
    ///
    /// It allocates `num_pages` pages from the page allocator.
    ///
    /// `align_pow2` must be a power of 2, and the returned region bound will be
    /// aligned to it.
    pub fn alloc_pages(
        &self,
        num_pages: usize,
        align_pow2: usize,
        kind: UsageKind,
    ) -> AllocResult<usize> {
        if !matches!(kind, UsageKind::RustHeap) {
            self.stats.lock().alloc(kind, num_pages * PAGE_SIZE);
        }
        self.palloc.lock().alloc_pages(num_pages, align_pow2)
    }

    /// Gives back the allocated pages starts from `pos` to the page allocator.
    ///
    /// The pages should be allocated by [`alloc_pages`], and `align_pow2`
    /// should be the same as the one used in [`alloc_pages`]. Otherwise, the
    /// behavior is undefined.
    ///
    /// [`alloc_pages`]: GlobalAllocator::alloc_pages
    pub fn dealloc_pages(&self, pos: usize, num_pages: usize, kind: UsageKind) {
        self.stats.lock().dealloc(kind, num_pages * PAGE_SIZE);
        self.palloc.lock().dealloc_pages(pos, num_pages)
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

    /// Returns the usage statistics of the allocator.
    pub fn usage_stats(&self) -> UsageStats {
        *self.stats.lock()
    }
}

#[cfg(feature = "tracking")]
mod tracking {
    use alloc::collections::btree_map::BTreeMap;
    use core::{
        alloc::Layout,
        ops::Range,
        sync::atomic::{AtomicBool, Ordering},
    };

    use axbacktrace::Backtrace;
    use kspin::SpinNoIrq;

    pub(crate) static TRACKING_ENABLED: AtomicBool = AtomicBool::new(false);

    #[percpu::def_percpu]
    pub(crate) static IN_GLOBAL_ALLOCATOR: bool = false;

    /// Metadata for each allocation made by the global allocator.
    #[derive(Debug)]
    pub struct AllocationInfo {
        pub layout: Layout,
        pub backtrace: Backtrace,
        pub generation: u64,
    }

    pub(crate) struct GlobalState {
        // FIXME: don't know why using HashMap causes crash
        pub map: BTreeMap<usize, AllocationInfo>,
        pub generation: u64,
    }

    static STATE: SpinNoIrq<GlobalState> = SpinNoIrq::new(GlobalState {
        map: BTreeMap::new(),
        generation: 0,
    });

    /// Enables allocation tracking.
    pub fn enable_tracking() {
        TRACKING_ENABLED.store(true, Ordering::SeqCst);
    }

    /// Disables allocation tracking.
    pub fn disable_tracking() {
        TRACKING_ENABLED.store(false, Ordering::SeqCst);
    }

    /// Returns whether allocation tracking is enabled.
    pub fn tracking_enabled() -> bool {
        TRACKING_ENABLED.load(Ordering::SeqCst)
    }

    pub(crate) fn with_state<R>(f: impl FnOnce(Option<&mut GlobalState>) -> R) -> R {
        IN_GLOBAL_ALLOCATOR.with_current(|in_global| {
            if *in_global || !tracking_enabled() {
                f(None)
            } else {
                *in_global = true;
                let mut state = STATE.lock();
                let result = f(Some(&mut state));
                *in_global = false;
                result
            }
        })
    }

    /// Returns the current generation of the global allocator.
    ///
    /// The generation is incremented every time a new allocation is made. It
    /// can be utilized to track the changes in the allocation state over time.
    ///
    /// See [`new_allocations_since`].
    pub fn current_generation() -> u64 {
        STATE.lock().generation
    }

    /// Visits all allocations made by the global allocator within the given
    /// generation range.
    pub fn allocations_in(range: Range<u64>, visitor: impl FnMut(&AllocationInfo)) {
        with_state(|state| {
            state
                .unwrap()
                .map
                .values()
                .filter(move |info| range.contains(&info.generation))
                .for_each(visitor)
        });
    }
}

#[cfg(feature = "tracking")]
pub use tracking::*;

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
            tracking::with_state(|state| match state {
                None => inner(),
                Some(state) => {
                    let ptr = inner();
                    let generation = state.generation;
                    state.generation += 1;
                    state.map.insert(
                        ptr as usize,
                        tracking::AllocationInfo {
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
        tracking::with_state(|state| match state {
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

#[cfg_attr(all(target_os = "none", not(test)), global_allocator)]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::new();

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

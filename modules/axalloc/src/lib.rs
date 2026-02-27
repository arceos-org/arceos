//! [ArceOS](https://github.com/arceos-org/arceos) global memory allocator.
//!
//! It provides [`GlobalAllocator`], which implements the trait
//! [`core::alloc::GlobalAlloc`]. A static global variable of type
//! [`GlobalAllocator`] is defined with the `#[global_allocator]` attribute, to
//! be registered as the standard library's default allocator.

#![no_std]

#[macro_use]
extern crate log;
extern crate alloc;

use core::fmt;
use strum::{IntoStaticStr, VariantArray};

const PAGE_SIZE: usize = 0x1000;

mod page;
pub use page::GlobalPage;

/// Tracking of memory usage, enabled with the `tracking` feature.
#[cfg(feature = "tracking")]
pub mod tracking;

/// Kinds of memory usage for tracking.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantArray, IntoStaticStr)]
pub enum UsageKind {
    /// Heap allocations made by kernel Rust code.
    RustHeap,
    /// Virtual memory, usually used for user space.
    VirtMem,
    /// Page cache for file systems.
    PageCache,
    /// Page tables.
    PageTable,
    /// DMA memory.
    Dma,
    /// Memory used by [`GlobalPage`].
    Global,
}

/// Statistics of memory usages.
#[derive(Clone, Copy)]
pub struct Usages([usize; UsageKind::VARIANTS.len()]);

impl Usages {
    const fn new() -> Self {
        Self([0; UsageKind::VARIANTS.len()])
    }

    fn alloc(&mut self, kind: UsageKind, size: usize) {
        self.0[kind as usize] += size;
    }

    fn dealloc(&mut self, kind: UsageKind, size: usize) {
        self.0[kind as usize] -= size;
    }

    /// Get the memory usage for a specific kind.
    pub fn get(&self, kind: UsageKind) -> usize {
        self.0[kind as usize]
    }
}

impl fmt::Debug for Usages {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("UsageStats");
        for &kind in UsageKind::VARIANTS {
            d.field(kind.into(), &self.0[kind as usize]);
        }
        d.finish()
    }
}

// Select implementation based on features
// When axvisor is enabled, use axvisor_impl (axallocator features are ignored)
#[cfg(feature = "hv")]
mod axvisor_impl;
#[cfg(feature = "hv")]
use axvisor_impl as imp;

// When axvisor is not enabled, use default_impl with axallocator
#[cfg(not(feature = "hv"))]
mod default_impl;
#[cfg(not(feature = "hv"))]
use default_impl as imp;

// Re-export types and functions from the implementation
pub use imp::{GlobalAllocator, global_add_memory, global_init};

// Re-export DefaultByteAllocator from both implementations
pub use imp::DefaultByteAllocator;

// Re-export AddrTranslator when using hv implementation
#[cfg(feature = "hv")]
pub use imp::AddrTranslator;

/// Returns the reference to the global allocator.
pub fn global_allocator() -> &'static GlobalAllocator {
    imp::global_allocator()
}

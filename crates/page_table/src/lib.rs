//! This crate provides generic, unified, architecture-independent, and OS-free
//! page table structures for various hardware architectures.
//!
//! The core struct is [`PageTable64<M, PTE, IF>`]. OS-functions and
//! architecture-dependent types are provided by generic parameters:
//!
//! - `M`: The architecture-dependent metadata, requires to implement
//!   the [`PagingMetaData`] trait.
//! - `PTE`: The architecture-dependent page table entry, requires to implement
//!   the [`GenericPTE`] trait.
//! - `IF`: OS-functions such as physical memory allocation, requires to
//!   implement the [`PagingIf`] trait.
//!
//! Currently supported architectures and page table structures:
//!
//! - x86: [`x86_64::X64PageTable`]
//! - ARM: [`aarch64::A64PageTable`]
//! - RISC-V: [`riscv::Sv39PageTable`], [`riscv::Sv48PageTable`]

#![no_std]
#![feature(const_trait_impl)]
#![feature(result_option_inspect)]
#![feature(doc_auto_cfg)]

#[macro_use]
extern crate log;

mod arch;
mod bits64;

use memory_addr::{PhysAddr, VirtAddr};

pub use self::arch::*;
pub use self::bits64::PageTable64;

#[doc(no_inline)]
pub use page_table_entry::{GenericPTE, MappingFlags};

/// The error type for page table operation failures.
#[derive(Debug)]
pub enum PagingError {
    /// Cannot allocate memory.
    NoMemory,
    /// The address is not aligned to the page size.
    NotAligned,
    /// The mapping is not present.
    NotMapped,
    /// The mapping is already present.
    AlreadyMapped,
    /// The page table entry represents a huge page, but the target physical
    /// frame is 4K in size.
    MappedToHugePage,
}

/// The specialized `Result` type for page table operations.
pub type PagingResult<T = ()> = Result<T, PagingError>;

/// The **architecture-dependent** metadata that must be provided for
/// [`PageTable64`].
#[const_trait]
pub trait PagingMetaData: Sync + Send + Sized {
    /// The number of levels of the hardware page table.
    const LEVELS: usize;
    /// The maximum number of bits of physical address.
    const PA_MAX_BITS: usize;
    /// The maximum number of bits of virtual address.
    const VA_MAX_BITS: usize;

    /// The maximum physical address.
    const PA_MAX_ADDR: usize = (1 << Self::PA_MAX_BITS) - 1;

    /// Whether a given physical address is valid.
    #[inline]
    fn paddr_is_valid(paddr: usize) -> bool {
        paddr <= Self::PA_MAX_ADDR // default
    }

    /// Whether a given virtual address is valid.
    #[inline]
    fn vaddr_is_valid(vaddr: usize) -> bool {
        // default: top bits sign extended
        let top_mask = usize::MAX << (Self::VA_MAX_BITS - 1);
        (vaddr & top_mask) == 0 || (vaddr & top_mask) == top_mask
    }
}

/// The low-level **OS-dependent** helpers that must be provided for
/// [`PageTable64`].
pub trait PagingIf: Sized {
    /// Request to allocate a 4K-sized physical frame.
    fn alloc_frame() -> Option<PhysAddr>;
    /// Request to free a allocated physical frame.
    fn dealloc_frame(paddr: PhysAddr);
    /// Returns a virtual address that maps to the given physical address.
    ///
    /// Used to access the physical memory directly in page table implementation.
    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr;
}

/// The page sizes supported by the hardware page table.
#[repr(usize)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PageSize {
    /// Size of 4 kilobytes (2<sup>12</sup> bytes).
    Size4K = 0x1000,
    /// Size of 2 megabytes (2<sup>21</sup> bytes).
    Size2M = 0x20_0000,
    /// Size of 1 gigabytes (2<sup>30</sup> bytes).
    Size1G = 0x4000_0000,
}

impl PageSize {
    /// Whether this page size is considered huge (larger than 4K).
    pub const fn is_huge(self) -> bool {
        matches!(self, Self::Size1G | Self::Size2M)
    }
}

impl From<PageSize> for usize {
    #[inline]
    fn from(size: PageSize) -> usize {
        size as usize
    }
}

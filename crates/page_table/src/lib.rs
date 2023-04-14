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

#[derive(Debug)]
pub enum PagingError {
    NoMemory,
    NotAligned,
    NotMapped,
    AlreadyMapped,
    MappedToHugePage,
}

pub type PagingResult<T = ()> = Result<T, PagingError>;

#[const_trait]
pub trait PagingMetaData: Sync + Send + Sized {
    const LEVELS: usize;
    const PA_MAX_BITS: usize;
    const VA_MAX_BITS: usize;

    const PA_MAX_ADDR: usize = (1 << Self::PA_MAX_BITS) - 1;

    #[inline]
    fn paddr_is_valid(paddr: usize) -> bool {
        paddr <= Self::PA_MAX_ADDR // default
    }

    #[inline]
    fn vaddr_is_valid(vaddr: usize) -> bool {
        // default: top bits sign extended
        let top_mask = usize::MAX << (Self::VA_MAX_BITS - 1);
        (vaddr & top_mask) == 0 || (vaddr & top_mask) == top_mask
    }
}

pub trait PagingIf: Sized {
    fn alloc_frame() -> Option<PhysAddr>;
    fn dealloc_frame(paddr: PhysAddr);
    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr;
}

#[repr(usize)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PageSize {
    Size4K = 0x1000,
    Size2M = 0x20_0000,
    Size1G = 0x4000_0000,
}

impl PageSize {
    pub const fn is_huge(self) -> bool {
        matches!(self, Self::Size1G | Self::Size2M)
    }
}

impl const From<PageSize> for usize {
    fn from(size: PageSize) -> usize {
        size as usize
    }
}

#![no_std]
#![feature(const_trait_impl)]
#![feature(result_option_inspect)]

#[macro_use]
extern crate log;

mod bits64;
mod pte;

use memory_addr::{PhysAddr, VirtAddr};

pub use bits64::PageTable64;
pub use pte::GenericPTE;

bitflags::bitflags! {
    pub struct MappingFlags: usize {
        const READ          = 1 << 0;
        const WRITE         = 1 << 1;
        const EXECUTE       = 1 << 2;
        const USER          = 1 << 3;
        const DEVICE        = 1 << 4;
    }
}

#[derive(Debug)]
pub enum PagingError {
    NoMemory,
    NotAligned,
    NotMapped,
    AlreadyMapped,
    MappedToHugePage,
}

pub type PagingResult<T = ()> = Result<T, PagingError>;

pub trait PageTableLevels: Sync + Send {
    const LEVELS: usize;
}
pub struct PageTableLevels3;
pub struct PageTableLevels4;

impl PageTableLevels for PageTableLevels3 {
    const LEVELS: usize = 3;
}

impl PageTableLevels for PageTableLevels4 {
    const LEVELS: usize = 4;
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

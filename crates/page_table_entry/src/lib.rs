#![no_std]
#![feature(const_trait_impl)]
#![feature(doc_auto_cfg)]
#![feature(doc_cfg)]

mod arch;

use core::fmt::Debug;
use memory_addr::PhysAddr;

pub use self::arch::*;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct MappingFlags: usize {
        const READ          = 1 << 0;
        const WRITE         = 1 << 1;
        const EXECUTE       = 1 << 2;
        const USER          = 1 << 3;
        const DEVICE        = 1 << 4;
    }
}

pub trait GenericPTE: Debug + Clone + Copy + Sync + Send + Sized {
    // Create a page table entry point to a terminate page or block.
    fn new_page(paddr: PhysAddr, flags: MappingFlags, is_huge: bool) -> Self;
    // Create a page table entry point to a next level page table.
    fn new_table(paddr: PhysAddr) -> Self;

    /// Returns the physical address mapped by this entry.
    fn paddr(&self) -> PhysAddr;
    /// Returns the flags of this entry.
    fn flags(&self) -> MappingFlags;
    /// Returns whether this entry is zero.
    fn is_unused(&self) -> bool;
    /// Returns whether this entry flag indicates present.
    fn is_present(&self) -> bool;
    /// For non-last level translation, returns whether this entry maps to a
    /// huge frame.
    fn is_huge(&self) -> bool;
    /// Set this entry to zero.
    fn clear(&mut self);
}

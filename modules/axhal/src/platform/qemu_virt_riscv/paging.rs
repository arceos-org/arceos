use core::fmt;

use page_table::{GenericPTE, PageTable64, PageTableLevels3};
use page_table::{PhysAddr, VirtAddr, PAGE_SIZE_4K};
use riscv::{asm, register::satp};

use crate::common::paging::PagingIfImpl;
pub use crate::common::paging::*;

bitflags::bitflags! {
    /// Page-table entry flags.
    pub struct PTEFlags: usize {
        /// Whether the PTE is valid.
        const V =   1 << 0;
        /// Whether the page is readable.
        const R =   1 << 1;
        /// Whether the page is writable.
        const W =   1 << 2;
        /// Whether the page is executable.
        const X =   1 << 3;
        /// Whether the page is accessible to user mode.
        const U =   1 << 4;
        /// Designates a global mapping.
        const G =   1 << 5;
        /// Indicates the virtual page has been read, written, or fetched from
        /// since the last time the A bit was cleared.
        const A =   1 << 6;
        /// Indicates the virtual page has been written since the last time the
        /// D bit was cleared.
        const D =   1 << 7;
    }
}

impl From<PTEFlags> for MappingFlags {
    fn from(f: PTEFlags) -> Self {
        let mut ret = Self::empty();
        if f.contains(PTEFlags::R) {
            ret |= Self::READ;
        }
        if f.contains(PTEFlags::W) {
            ret |= Self::WRITE;
        }
        if f.contains(PTEFlags::X) {
            ret |= Self::EXECUTE;
        }
        if f.contains(PTEFlags::U) {
            ret |= Self::USER;
        }
        ret
    }
}

impl From<MappingFlags> for PTEFlags {
    fn from(f: MappingFlags) -> Self {
        if f.is_empty() {
            return Self::empty();
        }
        let mut ret = Self::V;
        if f.contains(MappingFlags::READ) {
            ret |= Self::R;
        }
        if f.contains(MappingFlags::WRITE) {
            ret |= Self::W;
        }
        if f.contains(MappingFlags::EXECUTE) {
            ret |= Self::X;
        }
        if f.contains(MappingFlags::USER) {
            ret |= Self::U;
        }
        ret
    }
}

/// Sv39 and Sv48 page table entry.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Rv64PTE(u64);

impl PageTableEntry {
    const PHYS_ADDR_MASK: u64 = ((usize::MAX & !(PAGE_SIZE_4K - 1)) >> 2) as u64; // bits 10..54

    pub const fn empty() -> Self {
        Self(0)
    }
}

impl GenericPTE for Rv64PTE {
    fn new_page(paddr: PhysAddr, flags: MappingFlags, _is_block: bool) -> Self {
        let flags = PTEFlags::from(flags) | PTEFlags::A | PTEFlags::D;
        debug_assert!(flags.intersects(PTEFlags::R | PTEFlags::X));
        Self(flags.bits() as u64 | ((paddr >> 2) as u64 & Self::PHYS_ADDR_MASK))
    }
    fn new_table(paddr: PhysAddr) -> Self {
        Self(PTEFlags::V.bits() as u64 | ((paddr >> 2) as u64 & Self::PHYS_ADDR_MASK))
    }
    fn paddr(&self) -> PhysAddr {
        ((self.0 & Self::PHYS_ADDR_MASK) << 2) as PhysAddr
    }
    fn flags(&self) -> MappingFlags {
        PTEFlags::from_bits_truncate(self.0 as usize).into()
    }
    fn is_unused(&self) -> bool {
        self.0 == 0
    }
    fn is_present(&self) -> bool {
        PTEFlags::from_bits_truncate(self.0 as usize).contains(PTEFlags::V)
    }
    fn is_huge(&self) -> bool {
        PTEFlags::from_bits_truncate(self.0 as usize).intersects(PTEFlags::R | PTEFlags::X)
    }
    fn clear(&mut self) {
        self.0 = 0
    }
}

impl fmt::Debug for Rv64PTE {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("Rv64PTE");
        f.field("raw", &self.0)
            .field("paddr", &self.paddr())
            .field("flags", &self.flags())
            .finish()
    }
}

/// Sv39 and Sv48 page table entry.
pub type PageTableEntry = Rv64PTE;

/// Sv39: Page-Based 39-bit (3 levels) Virtual-Memory System.
pub type PageTable = PageTable64<PageTableLevels3, PageTableEntry, PagingIfImpl>;

pub fn read_page_table_root() -> PhysAddr {
    satp::read().ppn() << 12
}

pub unsafe fn write_page_table_root(root_paddr: PhysAddr) {
    let old_root = read_page_table_root();
    trace!("set page table root: {:#x} => {:#x}", old_root, root_paddr);
    if old_root != root_paddr {
        satp::set(satp::Mode::Sv39, 0, root_paddr >> 12);
        asm::sfence_vma_all();
    }
}

pub fn flush_tlb(vaddr: Option<VirtAddr>) {
    unsafe {
        if let Some(vaddr) = vaddr {
            asm::sfence_vma(0, vaddr)
        } else {
            asm::sfence_vma_all();
        }
    }
}

//! loongarch64 page table entries.

use crate::{GenericPTE, MappingFlags};
use core::fmt;
use memory_addr::PhysAddr;

bitflags::bitflags! {
    /// Page-table entry flags.
    #[derive(Debug)]
    pub struct PTEFlags: usize {
        /// Whether the PTE is valid.
        const V = 1 << 0;
        /// Indicates the virtual page has been written since the last time the
        /// D bit was cleared.
        const D = 1 << 1;
        /// Privilege Level with 2 bits.
        const PLVL = 1 << 2;
        const PLVH = 1 << 3;
        /// Memory Access Type controls the type of access, such as whether it
        /// can be cached by Cache, etc.
        const MATL = 1 << 4;
        const MATH = 1 << 5;
        /// Designates a global mapping OR Whether the page is huge page.
        const GH = 1 << 6;
        /// Whether the physical page is exist.
        const P = 1 << 7;
        /// Whether the page is writable.
        const W = 1 << 8;
        /// Designates a global mapping when using huge page.
        const G = 1 << 12;
        /// Whether the page is not readable.
        const NR = 1 << 61;
        /// Whether the page is not executable.
        const NX = 1 << 62;
        /// Whether the privilege Level is restricted. When RPLV is 0, the PTE
        /// can be accessed by any program with privilege Level highter than PLV.
        const RPLV = 1 << 63;
    }
}

impl From<PTEFlags> for MappingFlags {
    fn from(f: PTEFlags) -> Self {
        let mut ret = Self::empty();
        if !f.contains(PTEFlags::NR) {
            ret |= Self::READ;
        }
        if f.contains(PTEFlags::W) {
            ret |= Self::WRITE;
        }
        if !f.contains(PTEFlags::NX) {
            ret |= Self::EXECUTE;
        }
        if f.contains(PTEFlags::PLVL) {
            if f.contains(PTEFlags::PLVH) {
                ret |= Self::USER;
            }
        }
        if !f.contains(PTEFlags::MATL) {
            ret |= Self::DEVICE;
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
        if !f.contains(MappingFlags::READ) {
            ret |= Self::NR;
        }
        if f.contains(MappingFlags::WRITE) {
            ret |= Self::W;
        }
        if !f.contains(MappingFlags::EXECUTE) {
            ret |= Self::NX;
        }
        if f.contains(MappingFlags::USER) {
            ret |= Self::PLVH | Self::PLVL;
        }
        if !f.contains(MappingFlags::DEVICE) {
            ret |= Self::MATL;
        }
        ret
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct LA64PTE(u64);

impl LA64PTE {
    const PHYS_ADDR_MASK: u64 = 0x0000_ffff_ffff_f000; // bits 12..48
}

impl GenericPTE for LA64PTE {
    fn new_page(paddr: PhysAddr, flags: MappingFlags, _is_huge: bool) -> Self {
        let flags = PTEFlags::from(flags);
        Self(flags.bits() as u64 | ((paddr.as_usize()) as u64 & Self::PHYS_ADDR_MASK))
    }
    fn new_table(paddr: PhysAddr) -> Self {
        Self(PTEFlags::V.bits() as u64 | ((paddr.as_usize()) as u64 & Self::PHYS_ADDR_MASK))
    }
    fn paddr(&self) -> PhysAddr {
        PhysAddr::from((self.0 & Self::PHYS_ADDR_MASK) as usize)
    }
    fn flags(&self) -> MappingFlags {
        PTEFlags::from_bits_truncate(self.0 as usize).into()
    }

    fn set_paddr(&mut self, paddr: PhysAddr) {
        self.0 = (self.0 & !Self::PHYS_ADDR_MASK) | (paddr.as_usize() as u64 & Self::PHYS_ADDR_MASK)
    }

    fn set_flags(&mut self, flags: MappingFlags, _is_huge: bool) {
        let flags = PTEFlags::from(flags);
        self.0 = (self.0 & Self::PHYS_ADDR_MASK) | flags.bits() as u64;
    }

    fn is_unused(&self) -> bool {
        self.0 == 0
    }
    fn is_present(&self) -> bool {
        PTEFlags::from_bits_truncate(self.0 as usize).contains(PTEFlags::V)
    }
    fn is_huge(&self) -> bool {
        false
    }
    fn clear(&mut self) {
        self.0 = 0
    }
}

impl fmt::Debug for LA64PTE {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("LA64PTE");
        f.field("raw", &self.0)
            .field("paddr", &self.paddr())
            .field("flags", &self.flags())
            .field("is_unused", &self.is_unused())
            .field("is_present", &self.is_present())
            .field("is_huge", &self.is_huge())
            .finish()
    }
}

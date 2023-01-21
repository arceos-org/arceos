#![allow(unused_variables)]

pub mod console {
    pub fn putchar(c: u8) {
        unimplemented!()
    }

    pub fn getchar() -> Option<u8> {
        unimplemented!()
    }
}

pub mod misc {
    pub fn terminate() -> ! {
        unimplemented!()
    }
}

pub mod mem {
    pub use crate::common::mem::{memory_regions, phys_to_virt, virt_to_phys};
    pub use crate::common::mem::{MemRegion, MemRegionFlags};

    pub(crate) fn memory_regions_num() -> usize {
        0
    }

    pub(crate) fn memory_region_at(idx: usize) -> Option<MemRegion> {
        None
    }
}

#[cfg(feature = "paging")]
pub mod paging {
    pub use page_table::MappingFlags;
    use page_table::{PagingResult, PhysAddr, VirtAddr};

    pub struct PageTable;

    impl PageTable {
        pub const fn new() -> PagingResult<Self> {
            Ok(Self)
        }

        pub const fn root_paddr(&self) -> PhysAddr {
            0
        }

        pub fn map_region(
            &mut self,
            vaddr: VirtAddr,
            paddr: PhysAddr,
            size: usize,
            flags: MappingFlags,
            allow_huge: bool,
        ) -> PagingResult {
            unimplemented!()
        }

        pub fn unmap_region(&mut self, vaddr: VirtAddr, size: usize) -> PagingResult {
            unimplemented!()
        }
    }

    pub fn read_page_table_root() -> PhysAddr {
        0
    }
    pub unsafe fn write_page_table_root(root_paddr: PhysAddr) {}
    pub fn flush_tlb(vaddr: Option<VirtAddr>) {}
}

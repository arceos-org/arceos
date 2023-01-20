use crate::common::mem as common;

pub use common::{memory_regions, phys_to_virt, virt_to_phys, MemRegion, MemRegionFlags};

pub(crate) fn memory_regions_num() -> usize {
    common::common_memory_regions_num() + 1
}

pub(crate) fn memory_region_at(idx: usize) -> Option<MemRegion> {
    use core::cmp::Ordering;
    match idx.cmp(&common::common_memory_regions_num()) {
        Ordering::Less => common::common_memory_region_at(idx),
        Ordering::Equal => {
            // free memory
            extern "C" {
                fn ekernel();
            }
            let start = align_up(virt_to_phys(ekernel as usize), PAGE_SIZE);
            let end = align_down(axconfig::PHYS_MEMORY_END, PAGE_SIZE);
            Some(MemRegion {
                paddr: start,
                size: end - start,
                flags: MemRegionFlags::FREE | MemRegionFlags::READ | MemRegionFlags::WRITE,
                name: "free memory",
            })
        }
        Ordering::Greater => None,
    }
}

const PAGE_SIZE: usize = 0x1000;

const fn align_down(addr: usize, page_size: usize) -> usize {
    addr & !(page_size - 1)
}

const fn align_up(addr: usize, page_size: usize) -> usize {
    (addr + page_size - 1) & !(page_size - 1)
}

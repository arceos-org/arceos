use crate::common::mem::{common_memory_region_at, common_memory_regions_num};

pub use crate::common::mem::*;

pub(crate) const PA_MAX_BITS: usize = 56;
pub(crate) const VA_MAX_BITS: usize = 39; // Sv39

pub(crate) const PA_MAX_ADDR: usize = (1 << PA_MAX_BITS) - 1;

pub(crate) fn memory_regions_num() -> usize {
    common_memory_regions_num() + 1
}

pub(crate) fn memory_region_at(idx: usize) -> Option<MemRegion> {
    use core::cmp::Ordering;
    match idx.cmp(&common_memory_regions_num()) {
        Ordering::Less => common_memory_region_at(idx),
        Ordering::Equal => {
            // free memory
            extern "C" {
                fn ekernel();
            }
            let start = virt_to_phys((ekernel as usize).into()).align_up(PAGE_SIZE_4K);
            let end = PhysAddr::from(axconfig::PHYS_MEMORY_END).align_down(PAGE_SIZE_4K);
            Some(MemRegion {
                paddr: start,
                size: end.as_usize() - start.as_usize(),
                flags: MemRegionFlags::FREE | MemRegionFlags::READ | MemRegionFlags::WRITE,
                name: "free memory",
            })
        }
        Ordering::Greater => None,
    }
}

pub const fn paddr_is_valid(paddr: usize) -> bool {
    paddr <= PA_MAX_ADDR
}

pub const fn vaddr_is_valid(vaddr: usize) -> bool {
    // Top bits must be sign extended
    let top_mask = usize::MAX << (VA_MAX_BITS - 1);
    (vaddr & top_mask) == 0 || (vaddr & top_mask) == top_mask
}

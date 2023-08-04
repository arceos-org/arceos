use crate::mem::*;

/// Number of physical memory regions.
pub(crate) fn memory_regions_num() -> usize {
    #[cfg(feature = "test")]
    {
        common_memory_regions_num() + 2
    }

    #[cfg(not(feature = "test"))]
    {
        common_memory_regions_num() + 1
    }
}

#[cfg(feature = "test")]
fn extend_physical_memory(idx: usize) -> Option<MemRegion> {
    if idx == common_memory_regions_num() + 1 {
        let start = virt_to_phys(0xffff_ffc0_a000_0000.into()).align_up_4k();
        let end = PhysAddr::from(0xc000_0000).align_down_4k();
        Some(MemRegion {
            paddr: start,
            size: end.as_usize() - start.as_usize(),
            flags: MemRegionFlags::FREE | MemRegionFlags::READ | MemRegionFlags::WRITE,
            name: "free memory",
        })
    } else {
        None
    }
}

/// Returns the physical memory region at the given index, or [`None`] if the
/// index is out of bounds.
pub(crate) fn memory_region_at(idx: usize) -> Option<MemRegion> {
    use core::cmp::Ordering;
    match idx.cmp(&common_memory_regions_num()) {
        Ordering::Less => common_memory_region_at(idx),
        Ordering::Equal => {
            // free memory
            extern "C" {
                fn ekernel();
            }
            let start = virt_to_phys((ekernel as usize).into()).align_up_4k();
            let end = PhysAddr::from(axconfig::PHYS_MEMORY_END).align_down_4k();
            Some(MemRegion {
                paddr: start,
                size: end.as_usize() - start.as_usize(),
                flags: MemRegionFlags::FREE | MemRegionFlags::READ | MemRegionFlags::WRITE,
                name: "free memory",
            })
        }
        Ordering::Greater => {
            #[cfg(feature = "test")]
            {
                extend_physical_memory(idx)
            }
            #[cfg(not(feature = "test"))]
            None
        }
    }
}

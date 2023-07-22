use crate::mem::*;
use page_table_entry::{aarch64::A64PTE, GenericPTE, MappingFlags};

/// Number of physical memory regions.
pub(crate) fn memory_regions_num() -> usize {
    common_memory_regions_num() + 1
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
        Ordering::Greater => None,
    }
}

pub(crate) unsafe fn init_boot_page_table(
    boot_pt_l0: &mut [A64PTE; 512],
    boot_pt_l1: &mut [A64PTE; 512],
) {
    // 0x0000_0000_0000 ~ 0x0080_0000_0000, table
    boot_pt_l0[0] = A64PTE::new_table(PhysAddr::from(boot_pt_l1.as_ptr() as usize));
    // 0x0000_0000_0000..0x0000_4000_0000, 1G block, device memory
    boot_pt_l1[0] = A64PTE::new_page(
        PhysAddr::from(0),
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::DEVICE,
        true,
    );
    // 0x0000_4000_0000..0x0000_8000_0000, 1G block, normal memory
    boot_pt_l1[1] = A64PTE::new_page(
        PhysAddr::from(0x4000_0000),
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        true,
    );
}

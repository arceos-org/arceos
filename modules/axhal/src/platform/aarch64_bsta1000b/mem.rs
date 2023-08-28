use crate::mem::{MemRegion, PhysAddr};
use page_table_entry::{aarch64::A64PTE, GenericPTE, MappingFlags};

/// Returns platform-specific memory regions.
pub(crate) fn platform_regions() -> impl Iterator<Item = MemRegion> {
    crate::mem::default_free_regions().chain(crate::mem::default_mmio_regions())
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
    // 1G block, device memory
    boot_pt_l1[1] = A64PTE::new_page(
        PhysAddr::from(0x40000000),
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::DEVICE,
        true,
    );
    // 1G block, normal memory
    boot_pt_l1[2] = A64PTE::new_page(
        PhysAddr::from(0x80000000),
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        true,
    );
}

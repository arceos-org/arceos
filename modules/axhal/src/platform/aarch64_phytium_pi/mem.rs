use crate::mem::*;
use page_table_entry::{aarch64::A64PTE, GenericPTE, MappingFlags};

/// Returns platform-specific memory regions.
pub(crate) fn platform_regions() -> impl Iterator<Item = MemRegion> {
    core::iter::once(MemRegion {
        paddr: 0x0.into(),
        size: 0x1000,
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
        name: "spintable",
    })
    .chain(crate::mem::default_free_regions())
    .chain(crate::mem::default_mmio_regions())
}

pub(crate) unsafe fn init_boot_page_table(
    boot_pt_l0: *mut [A64PTE; 512],
    boot_pt_l1: *mut [A64PTE; 512],
) {
    let boot_pt_l0 = &mut *boot_pt_l0;
    let boot_pt_l1 = &mut *boot_pt_l1;
    // 0x0000_0000_0000 ~ 0x0080_0000_0000 - 1, table
    boot_pt_l0[0] = A64PTE::new_table(PhysAddr::from(boot_pt_l1.as_ptr() as usize));
    // 0x0000_0000_0000..0x0000_3fff_ffff, 1G block, device memory
    boot_pt_l1[0] = A64PTE::new_page(
        PhysAddr::from(0),
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::DEVICE,
        true,
    );
    // 0x8000_0000 ~ 0xbfff_ffff, 1G block, normay memory
    boot_pt_l1[2] = A64PTE::new_page(
        PhysAddr::from(0x8000_0000),
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        true,
    );
}

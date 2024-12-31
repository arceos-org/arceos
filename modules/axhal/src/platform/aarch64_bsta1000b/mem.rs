use crate::mem::{MemRegion, MemRegionFlags};
use page_table_entry::{GenericPTE, MappingFlags, aarch64::A64PTE};

/// Returns (a1000b only) memory regions.
pub(crate) fn default_a1000b_regions() -> impl Iterator<Item = MemRegion> {
    [MemRegion {
        paddr: pa!(0x80000000),
        size: 0x70000000,
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
        name: "reserved memory",
    }]
    .into_iter()
}
/// Returns platform-specific memory regions.
pub(crate) fn platform_regions() -> impl Iterator<Item = MemRegion> {
    crate::mem::default_free_regions()
        .chain(default_a1000b_regions())
        .chain(crate::mem::default_mmio_regions())
}

pub(crate) unsafe fn init_boot_page_table(
    boot_pt_l0: *mut [A64PTE; 512],
    boot_pt_l1: *mut [A64PTE; 512],
) {
    let boot_pt_l0 = &mut *boot_pt_l0;
    let boot_pt_l1 = &mut *boot_pt_l1;
    // 0x0000_0000_0000 ~ 0x0080_0000_0000, table
    boot_pt_l0[0] = A64PTE::new_table(pa!(boot_pt_l1.as_ptr() as usize));
    // 0x0000_0000_0000..0x0000_4000_0000, 1G block, device memory
    boot_pt_l1[0] = A64PTE::new_page(
        pa!(0),
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::DEVICE,
        true,
    );
    // 1G block, device memory
    boot_pt_l1[1] = A64PTE::new_page(
        pa!(0x40000000),
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::DEVICE,
        true,
    );
    // 1G block, normal memory
    boot_pt_l1[2] = A64PTE::new_page(
        pa!(0x80000000),
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        true,
    );
    boot_pt_l1[3] = A64PTE::new_page(
        pa!(0xc0000000),
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        true,
    );
    boot_pt_l1[6] = A64PTE::new_page(
        pa!(0x180000000),
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        true,
    );
    boot_pt_l1[7] = A64PTE::new_page(
        pa!(0x1C0000000),
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        true,
    );
}

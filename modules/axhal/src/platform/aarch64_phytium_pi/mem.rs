use crate::mem::*;
use page_table_entry::{aarch64::A64PTE, GenericPTE, MappingFlags};

/// Returns the default free memory regions (kernel image end to physical memory end).
fn __free_regions() -> impl Iterator<Item = MemRegion> {
    let start = virt_to_phys(
        VirtAddr::from(_ekernel as usize + axconfig::NOCACHE_MEMORY_SIZE).align_up_4k(),
    );
    let end = PhysAddr::from(axconfig::PHYS_MEMORY_END);
    core::iter::once(MemRegion {
        paddr: start,
        size: end.as_usize() - start.as_usize(),
        flags: MemRegionFlags::FREE | MemRegionFlags::READ | MemRegionFlags::WRITE,
        name: "free memory",
    })
}

/// Returns the default free memory regions (kernel image end to physical memory end).
fn __nocache_regions() -> impl Iterator<Item = MemRegion> {
    let start = VirtAddr::from(_ekernel as usize).align_up_4k();
    let start = virt_to_phys(start);

    core::iter::once(MemRegion {
        paddr: start,
        size: axconfig::NOCACHE_MEMORY_SIZE,
        flags: MemRegionFlags::DEVICE | MemRegionFlags::READ | MemRegionFlags::WRITE,
        name: "nocache memory",
    })
}


/// Returns platform-specific memory regions.
pub(crate) fn platform_regions() -> impl Iterator<Item = MemRegion> {
    core::iter::once(MemRegion {
        paddr: 0x0.into(),
        size: 0x1000,
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
        name: "spintable",
    })
    .chain(crate::mem::__nocache_regions())
    .chain(crate::mem::__free_regions())
    .chain(crate::mem::default_mmio_regions())
}

pub(crate) unsafe fn init_boot_page_table(
    boot_pt_l0: *mut [A64PTE; 512],
    boot_pt_l1: *mut [A64PTE; 512],
) {
    let boot_pt_l0 = &mut *boot_pt_l0;
    let boot_pt_l1 = &mut *boot_pt_l1;
    // 0x0000_0000_0000 ~ 0x0080_0000_0000, table
    boot_pt_l0[0] = A64PTE::new_table(PhysAddr::from(boot_pt_l1.as_ptr() as usize));
    // 0x0000_0000_0000..0x0000_8000_0000, 1G block, device memory
    boot_pt_l1[0] = A64PTE::new_page(
        PhysAddr::from(0),
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::DEVICE,
        true,
    );
    boot_pt_l1[2] = A64PTE::new_page(
        PhysAddr::from(0x8000_0000),
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        true,
    );
}

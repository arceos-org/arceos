// TODO: get memory regions from multiboot info.

use crate::mem::{MemRegion, MemRegionFlags};

/// Returns platform-specific memory regions.
pub(crate) fn platform_regions() -> impl Iterator<Item = MemRegion> {
    core::iter::once(MemRegion {
        paddr: pa!(0x1000),
        size: 0x9e000,
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
        name: "low memory",
    })
    .chain(crate::mem::default_free_regions())
    .chain(crate::mem::default_mmio_regions())
}

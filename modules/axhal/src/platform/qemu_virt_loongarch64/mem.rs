use crate::mem::MemRegion;

/// Returns platform-specific memory regions.
pub(crate) fn platform_regions() -> impl Iterator<Item = MemRegion> {
    crate::mem::default_free_regions().chain(crate::mem::default_mmio_regions())
}

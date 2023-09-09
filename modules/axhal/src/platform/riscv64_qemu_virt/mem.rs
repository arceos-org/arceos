use crate::mem::MemRegion;

/// Returns platform-specific memory regions.
pub(crate) fn platform_regions() -> impl Iterator<Item = MemRegion> {
    #[cfg(not(feature = "monolithic"))]
    return crate::mem::default_free_regions().chain(crate::mem::default_mmio_regions());

    #[cfg(feature = "monolithic")]
    return crate::mem::default_free_regions()
        .chain(crate::mem::default_mmio_regions())
        .chain(crate::mem::extend_free_regions());
}

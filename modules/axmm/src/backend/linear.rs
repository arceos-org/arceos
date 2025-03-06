use axhal::paging::{MappingFlags, PageTable};
use memory_addr::{PhysAddr, VirtAddr};

use super::Backend;

impl Backend {
    /// Creates a new linear mapping backend.
    pub const fn new_linear(pa_va_offset: usize) -> Self {
        Self::Linear { pa_va_offset }
    }

    pub(crate) fn map_linear(
        start: VirtAddr,
        size: usize,
        flags: MappingFlags,
        pt: &mut PageTable,
        pa_va_offset: usize,
    ) -> bool {
        let va_to_pa = |va: VirtAddr| PhysAddr::from(va.as_usize() - pa_va_offset);
        debug!(
            "map_linear: [{:#x}, {:#x}) -> [{:#x}, {:#x}) {:?}",
            start,
            start + size,
            va_to_pa(start),
            va_to_pa(start + size),
            flags
        );
        pt.map_region(start, va_to_pa, size, flags, false, false)
            .map(|tlb| tlb.ignore()) // TLB flush on map is unnecessary, as there are no outdated mappings.
            .is_ok()
    }

    pub(crate) fn unmap_linear(
        start: VirtAddr,
        size: usize,
        pt: &mut PageTable,
        _pa_va_offset: usize,
    ) -> bool {
        debug!("unmap_linear: [{:#x}, {:#x})", start, start + size);
        pt.unmap_region(start, size, true)
            .map(|tlb| tlb.ignore()) // flush each page on unmap, do not flush the entire TLB.
            .is_ok()
    }
}

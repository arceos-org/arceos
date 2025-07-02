use axhal::paging::{MappingFlags, PageTable};
use memory_addr::{PhysAddr, VirtAddr};

/// Linear mapping backend.
///
/// The offset between the virtual address and the physical address is
/// constant, which is specified by `pa_va_offset`. For example, the virtual
/// address `vaddr` is mapped to the physical address `vaddr - pa_va_offset`.
#[derive(Clone)]
pub struct Linear {
    offset: usize,
}

impl Linear {
    /// Creates a new linear mapping backend.
    pub(crate) const fn new(offset: usize) -> Self {
        Self { offset }
    }

    fn pa(&self, va: VirtAddr) -> PhysAddr {
        PhysAddr::from(va.as_usize().wrapping_sub(self.offset))
    }

    pub(crate) fn map(
        &self,
        start: VirtAddr,
        size: usize,
        flags: MappingFlags,
        pt: &mut PageTable,
    ) -> bool {
        debug!(
            "Linear::map [{:#x}, {:#x}) -> [{:#x}, {:#x}) {:?}",
            start,
            start + size,
            self.pa(start),
            self.pa(start + size),
            flags
        );
        pt.map_region(start, |va| self.pa(va), size, flags, false, false)
            .map(|tlb| tlb.ignore()) // TLB flush on map is unnecessary, as there are no outdated mappings.
            .is_ok()
    }

    pub(crate) fn unmap(&self, start: VirtAddr, size: usize, pt: &mut PageTable) -> bool {
        debug!("Linear::unmap [{:#x}, {:#x})", start, start + size);
        pt.unmap_region(start, size, true)
            .map(|tlb| tlb.ignore()) // flush each page on unmap, do not flush the entire TLB.
            .is_ok()
    }
}

use alloc::sync::Arc;

use axerrno::LinuxResult;
use axhal::paging::{MappingFlags, PageSize, PageTable};
use axsync::Mutex;
use memory_addr::{PhysAddr, PhysAddrRange, VirtAddr, VirtAddrRange};

use crate::{
    AddrSpace,
    backend::{Backend, BackendOps, paging_to_linux_error},
};

/// Linear mapping backend.
///
/// The offset between the virtual address and the physical address is
/// constant, which is specified by `pa_va_offset`. For example, the virtual
/// address `vaddr` is mapped to the physical address `vaddr - pa_va_offset`.
#[derive(Clone)]
pub struct LinearBackend {
    offset: isize,
}

impl LinearBackend {
    fn pa(&self, va: VirtAddr) -> PhysAddr {
        PhysAddr::from((va.as_usize() as isize - self.offset) as usize)
    }
}

impl BackendOps for LinearBackend {
    fn page_size(&self) -> PageSize {
        PageSize::Size4K
    }

    fn map(
        &self,
        range: VirtAddrRange,
        flags: MappingFlags,
        pt: &mut PageTable,
    ) -> LinuxResult<()> {
        let pa_range = PhysAddrRange::from_start_size(self.pa(range.start), range.size());
        debug!("Linear::map: {range:?} -> {pa_range:?} {flags:?}");
        pt.map_region(range.start, |va| self.pa(va), range.size(), flags, false)
            .map_err(paging_to_linux_error)
    }

    fn unmap(&self, range: VirtAddrRange, pt: &mut PageTable) -> LinuxResult<()> {
        let pa_range = PhysAddrRange::from_start_size(self.pa(range.start), range.size());
        debug!("Linear::unmap: {range:?} -> {pa_range:?}");
        pt.unmap_region(range.start, range.size())
            .map_err(paging_to_linux_error)
    }

    fn clone_map(
        &self,
        _range: VirtAddrRange,
        _flags: MappingFlags,
        _old_pt: &mut PageTable,
        _new_pt: &mut PageTable,
        _new_aspace: &Arc<Mutex<AddrSpace>>,
    ) -> LinuxResult<Backend> {
        Ok(Backend::Linear(self.clone()))
    }
}

impl Backend {
    pub fn new_linear(offset: isize) -> Self {
        Self::Linear(LinearBackend { offset })
    }
}

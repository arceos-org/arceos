//! Memory mapping backends.

use axerrno::{AxError, AxResult};
use axhal::paging::{MappingFlags, PageTable, PagingError, PagingResult};
use memory_addr::VirtAddr;
use memory_set::MappingBackend;

mod alloc;
mod linear;

/// A unified enum type for different memory mapping backends.
#[derive(Clone)]
pub enum Backend {
    /// Linear mappings. The target physical frames are contiguous and their
    /// addresses should be known when creating the mapping.
    Linear(linear::Linear),
    /// Lazy mappings. The target physical frames are obtained from the global
    /// allocator.
    Alloc(alloc::Alloc),
}

impl MappingBackend for Backend {
    type Addr = VirtAddr;
    type Flags = MappingFlags;
    type PageTable = PageTable;

    fn map(&self, start: VirtAddr, size: usize, flags: MappingFlags, pt: &mut PageTable) -> bool {
        match self {
            Self::Linear(linear) => linear.map(start, size, flags, pt),
            Self::Alloc(alloc) => alloc.map(start, size, flags, pt),
        }
    }

    fn unmap(&self, start: VirtAddr, size: usize, pt: &mut PageTable) -> bool {
        match self {
            Self::Linear(linear) => linear.unmap(start, size, pt),
            Self::Alloc(alloc) => alloc.unmap(start, size, pt),
        }
    }

    fn protect(
        &self,
        start: Self::Addr,
        size: usize,
        new_flags: Self::Flags,
        pt: &mut Self::PageTable,
    ) -> bool {
        pt.protect_region(start, size, new_flags, true)
            .map(|tlb| tlb.ignore())
            .is_ok()
    }
}

impl Backend {
    pub(crate) fn new_linear(offset: usize) -> Self {
        Self::Linear(linear::Linear::new(offset))
    }

    pub(crate) fn new_alloc(populate: bool) -> Self {
        Self::Alloc(alloc::Alloc::new(populate))
    }

    pub(crate) fn populate_strict(
        &self,
        start: VirtAddr,
        size: usize,
        flags: MappingFlags,
        pt: &mut PageTable,
    ) -> PagingResult {
        match self {
            Self::Linear(_) => Err(PagingError::AlreadyMapped),
            Self::Alloc(alloc) => alloc.populate(start, size, flags, pt),
        }
    }

    pub(crate) fn populate_safe(
        &self,
        start: VirtAddr,
        size: usize,
        flags: MappingFlags,
        pt: &mut PageTable,
    ) -> AxResult {
        match self.populate_strict(start, size, flags, pt) {
            Ok(()) | Err(PagingError::AlreadyMapped) => Ok(()),
            Err(PagingError::NoMemory) => Err(AxError::NoMemory),
            Err(_) => Err(AxError::BadAddress),
        }
    }

    pub(crate) fn needs_copying(&self) -> bool {
        matches!(self, Self::Alloc(_))
    }
}

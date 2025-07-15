//! Memory mapping backends.
use ::alloc::sync::Arc;
use axalloc::{UsageKind, global_allocator};
use axerrno::{LinuxError, LinuxResult};
use axhal::{
    mem::{phys_to_virt, virt_to_phys},
    paging::{MappingFlags, PageTable, PagingError, PagingResult},
};
use memory_addr::{PAGE_SIZE_4K, PhysAddr, VirtAddr};
use memory_set::MappingBackend;

mod alloc;
mod linear;
mod shared;

pub use shared::SharedPages;

fn alloc_frame(zeroed: bool) -> Option<PhysAddr> {
    let vaddr = VirtAddr::from(
        global_allocator()
            .alloc_pages(1, PAGE_SIZE_4K, UsageKind::UserMem)
            .ok()?,
    );
    if zeroed {
        unsafe { core::ptr::write_bytes(vaddr.as_mut_ptr(), 0, PAGE_SIZE_4K) };
    }
    let paddr = virt_to_phys(vaddr);
    Some(paddr)
}

fn dealloc_frame(frame: PhysAddr) {
    let vaddr = phys_to_virt(frame);
    global_allocator().dealloc_pages(vaddr.as_usize(), 1, UsageKind::UserMem);
}

/// A unified enum type for different memory mapping backends.
#[derive(Clone)]
pub enum Backend {
    /// Linear mappings. The target physical frames are contiguous and their
    /// addresses should be known when creating the mapping.
    Linear(linear::Linear),
    /// Lazy mappings. The target physical frames are obtained from the global
    /// allocator.
    Alloc(alloc::Alloc),
    Shared(shared::Shared),
}

impl MappingBackend for Backend {
    type Addr = VirtAddr;
    type Flags = MappingFlags;
    type PageTable = PageTable;

    fn map(&self, start: VirtAddr, size: usize, flags: MappingFlags, pt: &mut PageTable) -> bool {
        match self {
            Self::Linear(linear) => linear.map(start, size, flags, pt),
            Self::Alloc(alloc) => alloc.map(start, size, flags, pt),
            Self::Shared(shared) => shared.map(start, flags, pt),
        }
        .is_ok()
    }

    fn unmap(&self, start: VirtAddr, size: usize, pt: &mut PageTable) -> bool {
        match self {
            Self::Linear(linear) => linear.unmap(start, size, pt),
            Self::Alloc(alloc) => alloc.unmap(start, size, pt),
            Self::Shared(shared) => shared.unmap(start, pt),
        }
        .is_ok()
    }

    fn protect(
        &self,
        start: Self::Addr,
        size: usize,
        new_flags: Self::Flags,
        pt: &mut Self::PageTable,
    ) -> bool {
        pt.protect_region(start, size, new_flags).is_ok()
    }
}

impl Backend {
    pub(crate) fn new_linear(offset: usize) -> Self {
        Self::Linear(linear::Linear::new(offset))
    }

    pub(crate) fn new_alloc(populate: bool) -> Self {
        Self::Alloc(alloc::Alloc::new(populate))
    }

    pub(crate) fn new_shared(pages: Arc<SharedPages>) -> Self {
        Self::Shared(shared::Shared { pages })
    }

    pub(crate) fn populate_strict(
        &self,
        start: VirtAddr,
        size: usize,
        flags: MappingFlags,
        pt: &mut PageTable,
    ) -> PagingResult {
        match self {
            Self::Linear(_) | Self::Shared(_) => Err(PagingError::AlreadyMapped),
            Self::Alloc(alloc) => alloc.populate(start, size, flags, pt),
        }
    }

    pub(crate) fn populate_safe(
        &self,
        start: VirtAddr,
        size: usize,
        flags: MappingFlags,
        pt: &mut PageTable,
    ) -> LinuxResult {
        match self.populate_strict(start, size, flags, pt) {
            Ok(()) | Err(PagingError::AlreadyMapped) => Ok(()),
            Err(PagingError::NoMemory) => Err(LinuxError::ENOMEM),
            Err(_) => Err(LinuxError::EFAULT),
        }
    }

    pub(crate) fn needs_copying(&self) -> bool {
        matches!(self, Self::Alloc(_))
    }

    pub fn pages(&self) -> Option<Arc<SharedPages>> {
        match self {
            Self::Shared(shared) => Some(shared.pages.clone()),
            _ => None,
        }
    }
}

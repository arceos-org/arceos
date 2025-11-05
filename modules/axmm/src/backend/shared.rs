use alloc::{sync::Arc, vec::Vec};
use core::ops::Deref;

use axerrno::AxResult;
use axhal::paging::{MappingFlags, PageSize, PageTableMut};
use axsync::Mutex;
use memory_addr::{MemoryAddr, PhysAddr, VirtAddr, VirtAddrRange};

use super::{alloc_frame, dealloc_frame};
use crate::{
    AddrSpace,
    backend::{Backend, BackendOps, divide_page, pages_in},
};

pub struct SharedPages {
    pub phys_pages: Vec<PhysAddr>,
    pub size: PageSize,
}
impl SharedPages {
    pub fn new(size: usize, page_size: PageSize) -> AxResult<Self> {
        Ok(Self {
            phys_pages: (0..divide_page(size, page_size))
                .map(|_| alloc_frame(true, page_size))
                .collect::<AxResult<_>>()?,
            size: page_size,
        })
    }

    pub fn len(&self) -> usize {
        self.phys_pages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.phys_pages.is_empty()
    }
}

impl Deref for SharedPages {
    type Target = [PhysAddr];

    fn deref(&self) -> &Self::Target {
        &self.phys_pages
    }
}

impl Drop for SharedPages {
    fn drop(&mut self) {
        for frame in &self.phys_pages {
            dealloc_frame(*frame, self.size);
        }
    }
}

// FIXME: This implementation does not allow map or unmap partial ranges.
#[derive(Clone)]
pub struct SharedBackend {
    start: VirtAddr,
    pages: Arc<SharedPages>,
}
impl SharedBackend {
    pub fn pages(&self) -> &Arc<SharedPages> {
        &self.pages
    }

    fn pages_starting_from(&self, start: VirtAddr) -> &[PhysAddr] {
        debug_assert!(start.is_aligned(self.pages.size));
        let start_index = divide_page(start - self.start, self.pages.size);
        &self.pages[start_index..]
    }
}

impl BackendOps for SharedBackend {
    fn page_size(&self) -> PageSize {
        self.pages.size
    }

    fn map(&self, range: VirtAddrRange, flags: MappingFlags, pt: &mut PageTableMut) -> AxResult {
        debug!("Shared::map: {:?} {:?}", range, flags);
        for (vaddr, paddr) in
            pages_in(range, self.pages.size)?.zip(self.pages_starting_from(range.start))
        {
            pt.map(vaddr, *paddr, self.pages.size, flags)?;
        }
        Ok(())
    }

    fn unmap(&self, range: VirtAddrRange, pt: &mut PageTableMut) -> AxResult {
        debug!("Shared::unmap: {:?}", range);
        for vaddr in pages_in(range, self.pages.size)? {
            pt.unmap(vaddr)?;
        }
        Ok(())
    }

    fn clone_map(
        &self,
        _range: VirtAddrRange,
        _flags: MappingFlags,
        _old_pt: &mut PageTableMut,
        _new_pt: &mut PageTableMut,
        _new_aspace: &Arc<Mutex<AddrSpace>>,
    ) -> AxResult<Backend> {
        Ok(Backend::Shared(self.clone()))
    }
}

impl Backend {
    pub fn new_shared(start: VirtAddr, pages: Arc<SharedPages>) -> Self {
        Self::Shared(SharedBackend { start, pages })
    }
}

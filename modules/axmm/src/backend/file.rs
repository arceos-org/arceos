use alloc::{
    boxed::Box,
    sync::{Arc, Weak},
    vec::Vec,
};
use core::sync::atomic::{AtomicUsize, Ordering};

use axerrno::{AxError, AxResult};
use axfs_ng::{CachedFile, FileFlags};
use axhal::paging::{MappingFlags, PageSize, PageTableMut, PagingError};
use axsync::Mutex;
use memory_addr::{PAGE_SIZE_4K, VirtAddr, VirtAddrRange};

use crate::{
    AddrSpace,
    backend::{Backend, BackendOps, pages_in, paging_to_ax_error},
};

#[doc(hidden)]
pub struct FileBackendInner {
    start: VirtAddr,
    cache: CachedFile,
    flags: FileFlags,
    offset_page: u32,
    handle: AtomicUsize,
    futex_handle: Arc<()>,
}
impl Drop for FileBackendInner {
    fn drop(&mut self) {
        let handle = self.handle.load(Ordering::Acquire);
        if handle != 0 {
            unsafe {
                self.cache.remove_evict_listener(handle);
            }
        }
    }
}
impl FileBackendInner {
    pub fn register_listener(self: &Arc<Self>, aspace: &Arc<Mutex<AddrSpace>>) -> usize {
        let aspace = Arc::downgrade(aspace);
        self.cache.add_evict_listener({
            let this = Arc::downgrade(self);
            move |pn, _page| {
                let Some(this) = this.upgrade() else {
                    return;
                };
                let Some(aspace) = aspace.upgrade() else {
                    // The address space has been dropped, nothing to do.
                    return;
                };
                let Some(mut aspace) = aspace.try_lock() else {
                    // This can happen during the populate process, when new pages
                    // are being populated and old pages are being evicted. In this
                    // case, we delegate the unmapping to the populate process.
                    return;
                };
                this.on_evict(pn, &mut aspace);
            }
        })
    }

    fn on_evict(self: &Arc<Self>, pn: u32, aspace: &mut AddrSpace) {
        let Some(pn) = pn.checked_sub(self.offset_page) else {
            return;
        };
        let vaddr = self.start + pn as usize * PageSize::Size4K as usize;
        if !aspace.find_area(vaddr).is_some_and(
            |it| matches!(it.backend(), Backend::File(file) if Arc::ptr_eq(&file.0, self)),
        ) {
            // Ignore if the page is not controlled by this file mapping.
            return;
        }

        let pt = aspace.page_table_mut();
        match pt.to_mut().unmap(vaddr) {
            Ok(_) | Err(PagingError::NotMapped) => {}
            Err(err) => {
                warn!("Failed to unmap page {:?}: {:?}", vaddr, err);
            }
        }
    }
}

/// File-backed mapping backend.
#[derive(Clone)]
pub struct FileBackend(Arc<FileBackendInner>);
impl FileBackend {
    fn check_flags(&self, flags: MappingFlags) -> AxResult {
        let mut required_flags = FileFlags::empty();
        if flags.contains(MappingFlags::READ) {
            required_flags |= FileFlags::READ;
        }
        if flags.contains(MappingFlags::WRITE) {
            required_flags |= FileFlags::WRITE;
        }

        if !self.0.flags.contains(required_flags) {
            return Err(AxError::PermissionDenied);
        }
        Ok(())
    }

    pub fn futex_handle(&self) -> Weak<()> {
        Arc::downgrade(&self.0.futex_handle)
    }
}

impl BackendOps for FileBackend {
    fn page_size(&self) -> PageSize {
        PageSize::Size4K
    }

    fn map(&self, _range: VirtAddrRange, flags: MappingFlags, _pt: &mut PageTableMut) -> AxResult {
        self.check_flags(flags)
    }

    fn unmap(&self, range: VirtAddrRange, pt: &mut PageTableMut) -> AxResult {
        for addr in pages_in(range, PageSize::Size4K)? {
            match pt.unmap(addr) {
                Ok(_) | Err(PagingError::NotMapped) => {}
                Err(err) => {
                    warn!("Failed to unmap page {:?}: {:?}", addr, err);
                    return Err(paging_to_ax_error(err));
                }
            }
        }
        Ok(())
    }

    fn on_protect(
        &self,
        _range: VirtAddrRange,
        new_flags: MappingFlags,
        _pt: &mut PageTableMut,
    ) -> AxResult {
        self.check_flags(new_flags)
    }

    fn populate(
        &self,
        range: VirtAddrRange,
        flags: MappingFlags,
        access_flags: MappingFlags,
        pt: &mut PageTableMut,
    ) -> AxResult<(usize, Option<Box<dyn FnOnce(&mut AddrSpace)>>)> {
        let mut pages = 0;
        let mut to_be_evicted = Vec::new();
        let start_page = ((range.start - self.0.start) / PAGE_SIZE_4K) as u32 + self.0.offset_page;
        for (i, addr) in pages_in(range, PageSize::Size4K)?.enumerate() {
            let pn = start_page + i as u32;
            match pt.query(addr) {
                Ok((paddr, page_flags, _)) => {
                    if access_flags.contains(MappingFlags::WRITE)
                        && !page_flags.contains(MappingFlags::WRITE)
                    {
                        let in_memory = self.0.cache.in_memory();
                        self.0.cache.with_page(pn, |page| {
                            if !in_memory {
                                page.expect("page should be present").mark_dirty();
                            }
                            pt.remap(addr, paddr, flags).map_err(paging_to_ax_error)?;
                            pages += 1;
                            Ok(())
                        })?;
                    }
                }
                // If the page is not mapped, try map it.
                Err(PagingError::NotMapped) => {
                    let map_flags = if self.0.cache.in_memory() {
                        // For in memory files, we don't need to (and also
                        // musn't) mark them dirty, so we can use the original
                        // flags.
                        flags
                    } else {
                        flags - MappingFlags::WRITE
                    };
                    self.0.cache.with_page_or_insert(pn, |page, evicted| {
                        if let Some((pn, _)) = evicted {
                            to_be_evicted.push(pn);
                        }
                        pt.map(addr, page.paddr(), PageSize::Size4K, map_flags)
                            .map_err(paging_to_ax_error)?;
                        pages += 1;
                        Ok(())
                    })?;
                }
                Err(_) => return Err(AxError::BadAddress),
            }
        }
        Ok((
            pages,
            if to_be_evicted.is_empty() {
                None
            } else {
                let inner = self.0.clone();
                Some(Box::new(move |aspace: &mut AddrSpace| {
                    for pn in to_be_evicted {
                        inner.on_evict(pn, aspace);
                    }
                }))
            },
        ))
    }

    fn clone_map(
        &self,
        _range: VirtAddrRange,
        _flags: MappingFlags,
        _old_pt: &mut PageTableMut,
        _new_pt: &mut PageTableMut,
        new_aspace: &Arc<Mutex<AddrSpace>>,
    ) -> AxResult<Backend> {
        let inner = Arc::new(FileBackendInner {
            start: self.0.start,
            cache: self.0.cache.clone(),
            flags: self.0.flags,
            offset_page: self.0.offset_page,
            handle: AtomicUsize::new(0),
            futex_handle: self.0.futex_handle.clone(),
        });
        inner.register_listener(new_aspace);
        Ok(Backend::File(FileBackend(inner)))
    }
}

impl Backend {
    pub fn new_file(
        start: VirtAddr,
        cache: CachedFile,
        flags: FileFlags,
        offset: usize,
        aspace: &Arc<Mutex<AddrSpace>>,
    ) -> Self {
        let offset_page = (offset / PAGE_SIZE_4K) as u32;
        let inner = Arc::new(FileBackendInner {
            start,
            cache,
            flags,
            offset_page,
            handle: AtomicUsize::new(0),
            futex_handle: Arc::new(()),
        });
        inner.register_listener(aspace);
        Self::File(FileBackend(inner))
    }
}

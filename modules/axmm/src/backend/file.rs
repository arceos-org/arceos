use alloc::{boxed::Box, sync::Arc, vec::Vec};

use axerrno::{LinuxError, LinuxResult};
use axfs_ng::{CachedFile, FileFlags};
use axhal::paging::{MappingFlags, PageSize, PageTable};
use axsync::{Mutex, RawMutex};
use memory_addr::{PAGE_SIZE_4K, VirtAddr, VirtAddrRange};
use page_table_multiarch::PagingError;

use crate::{
    AddrSpace, Backend,
    backend::{BackendOps, pages_in, paging_to_linux_error},
};

/// File-backed mapping backend.
pub struct FileBackend {
    start: VirtAddr,
    cache: Arc<CachedFile<RawMutex>>,
    flags: FileFlags,
    offset_page: u32,
    handle: Option<usize>,
}
impl Clone for FileBackend {
    fn clone(&self) -> Self {
        Self {
            start: self.start,
            cache: self.cache.clone(),
            flags: self.flags,
            offset_page: self.offset_page,
            // We only need one evict listener for multiple mappings within one
            // address space
            handle: None,
        }
    }
}
impl Drop for FileBackend {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            unsafe {
                self.cache.remove_evict_listener(handle);
            }
        }
    }
}

impl FileBackend {
    fn check_flags(&self, flags: MappingFlags) -> LinuxResult<()> {
        let mut required_flags = FileFlags::empty();
        if flags.contains(MappingFlags::READ) {
            required_flags |= FileFlags::READ;
        }
        if flags.contains(MappingFlags::WRITE) {
            required_flags |= FileFlags::WRITE;
        }

        if !self.flags.contains(required_flags) {
            return Err(LinuxError::EACCES);
        }
        Ok(())
    }
}

impl BackendOps for FileBackend {
    fn page_size(&self) -> PageSize {
        PageSize::Size4K
    }

    fn map(
        &self,
        _range: VirtAddrRange,
        flags: MappingFlags,
        _pt: &mut PageTable,
    ) -> LinuxResult<()> {
        self.check_flags(flags)
    }

    fn unmap(&self, range: VirtAddrRange, pt: &mut PageTable) -> LinuxResult<()> {
        for addr in pages_in(range, PageSize::Size4K)? {
            match pt.unmap(addr) {
                Ok(_) | Err(PagingError::NotMapped) => {}
                Err(err) => {
                    warn!("Failed to unmap page {:?}: {:?}", addr, err);
                    return Err(paging_to_linux_error(err));
                }
            }
        }
        Ok(())
    }

    fn on_protect(
        &self,
        _range: VirtAddrRange,
        new_flags: MappingFlags,
        _pt: &mut PageTable,
    ) -> LinuxResult<()> {
        self.check_flags(new_flags)
    }

    fn populate(
        &self,
        range: VirtAddrRange,
        flags: MappingFlags,
        access_flags: MappingFlags,
        pt: &mut PageTable,
    ) -> LinuxResult<(usize, Option<Box<dyn FnOnce(&mut AddrSpace)>>)> {
        let mut pages = 0;
        let mut to_be_evicted = Vec::new();
        let start_page = ((range.start - self.start) / PAGE_SIZE_4K) as u32 + self.offset_page;
        for (i, addr) in pages_in(range, PageSize::Size4K)?.enumerate() {
            let pn = start_page + i as u32;
            match pt.query(addr) {
                Ok((paddr, page_flags, _)) => {
                    if access_flags.contains(MappingFlags::WRITE)
                        && !page_flags.contains(MappingFlags::WRITE)
                    {
                        self.cache.with_page(pn, |page| {
                            page.expect("page should be present").mark_dirty();
                            pt.remap(addr, paddr, flags)
                                .map_err(paging_to_linux_error)?;
                            pages += 1;
                            Ok(())
                        })?;
                    }
                }
                // If the page is not mapped, try map it.
                Err(PagingError::NotMapped) => {
                    self.cache.with_page_or_insert(pn, |page, evicted| {
                        if let Some((pn, _)) = evicted {
                            to_be_evicted.push(pn);
                        }
                        pt.map(
                            addr,
                            page.paddr(),
                            PageSize::Size4K,
                            flags - MappingFlags::WRITE,
                        )
                        .map_err(paging_to_linux_error)?;
                        pages += 1;
                        Ok(())
                    })?;
                }
                Err(_) => return Err(LinuxError::EFAULT),
            }
        }
        Ok((
            pages,
            if to_be_evicted.is_empty() {
                None
            } else {
                let cache = self.cache.clone();
                Some(Box::new(move |aspace: &mut AddrSpace| {
                    let start = range.start;
                    for pn in to_be_evicted {
                        on_evict(aspace, start, &cache, pn);
                    }
                })
                    as Box<dyn FnOnce(&mut AddrSpace) + 'static>)
            },
        ))
    }

    fn clone_map(
        &self,
        _range: VirtAddrRange,
        _flags: MappingFlags,
        _old_pt: &mut PageTable,
        _new_pt: &mut PageTable,
    ) -> LinuxResult<Backend> {
        Ok(Backend::File(self.clone()))
    }
}

fn on_evict(aspace: &mut AddrSpace, start: VirtAddr, cache: &Arc<CachedFile<RawMutex>>, pn: u32) {
    let vaddr = start + pn as usize * PageSize::Size4K as usize;
    if !aspace.find_area(vaddr).is_some_and(
        |it| matches!(it.backend(), Backend::File(file) if Arc::ptr_eq(&file.cache, cache)),
    ) {
        // Ignore if the page is not controlled by this file mapping.
        return;
    }

    let pt = aspace.page_table_mut();
    match pt.unmap(vaddr) {
        Ok(_) | Err(PagingError::NotMapped) => {}
        Err(err) => {
            warn!("Failed to unmap page {:?}: {:?}", vaddr, err);
        }
    }
}

impl Backend {
    pub fn new_file(
        start: VirtAddr,
        cache: Arc<CachedFile<RawMutex>>,
        flags: FileFlags,
        offset: usize,
        aspace: Arc<Mutex<AddrSpace>>,
    ) -> Self {
        let handle = cache.add_evict_listener({
            let cache = cache.clone();
            move |pn, _page| {
                let Some(mut aspace) = aspace.try_lock() else {
                    // This can happen during the populate process, when new pages
                    // are being populated and old pages are being evicted. In this
                    // case, we delegate the unmapping to the populate process.
                    return;
                };
                on_evict(&mut aspace, start, &cache, pn);
            }
        });
        let offset_page = offset / PAGE_SIZE_4K;
        Self::File(FileBackend {
            start,
            cache,
            flags,
            offset_page: offset_page as u32,
            handle: Some(handle),
        })
    }
}

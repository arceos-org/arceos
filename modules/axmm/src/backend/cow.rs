use alloc::{boxed::Box, sync::Arc};
use core::slice;

use axerrno::{LinuxError, LinuxResult};
use axfs_ng::FileBackend;
use axhal::{
    mem::phys_to_virt,
    paging::{MappingFlags, PageSize, PageTable, PagingError},
};
use axsync::{Mutex, RawMutex};
use memory_addr::{PhysAddr, VirtAddr, VirtAddrRange};

use crate::{
    AddrSpace, Backend,
    backend::{BackendOps, alloc_frame, dealloc_frame, pages_in, paging_to_linux_error},
    page_info::frame_table,
};

/// Copy-on-write mapping backend.
///
/// This corresponds to the `MAP_PRIVATE` flag.
#[derive(Clone)]
pub struct CowBackend {
    start: VirtAddr,
    size: PageSize,
    file: Option<(FileBackend<RawMutex>, usize)>,
}

impl CowBackend {
    fn alloc_new_at(
        &self,
        vaddr: VirtAddr,
        flags: MappingFlags,
        pt: &mut PageTable,
    ) -> LinuxResult<()> {
        let frame = alloc_frame(true, self.size)?;
        frame_table().inc_ref(frame);
        if let Some((file, offset)) = &self.file {
            let buf = unsafe {
                slice::from_raw_parts_mut(phys_to_virt(frame).as_mut_ptr(), self.size as _)
            };
            file.read_at(buf, (vaddr - self.start) as u64 + *offset as u64)?;
        }
        pt.map(vaddr, frame, self.size, flags)
            .map_err(paging_to_linux_error)?;
        Ok(())
    }

    fn handle_cow_fault(
        &self,
        vaddr: VirtAddr,
        paddr: PhysAddr,
        flags: MappingFlags,
        pt: &mut PageTable,
    ) -> LinuxResult<()> {
        match frame_table().ref_count(paddr) {
            0 => unreachable!(),
            // There is only one AddrSpace reference to the page,
            // so there is no need to copy it.
            1 => {
                pt.protect(vaddr, flags).map_err(paging_to_linux_error)?;
            }
            // Allocates the new page and copies the contents of the original page,
            // remapping the virtual address to the physical address of the new page.
            2.. => {
                frame_table().dec_ref(paddr);

                let new_frame = alloc_frame(false, self.size)?;
                frame_table().inc_ref(new_frame);
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        phys_to_virt(paddr).as_ptr(),
                        phys_to_virt(new_frame).as_mut_ptr(),
                        self.size as _,
                    );
                }

                pt.remap(vaddr, new_frame, flags)
                    .map_err(paging_to_linux_error)?;
            }
        }

        Ok(())
    }
}

impl BackendOps for CowBackend {
    fn page_size(&self) -> PageSize {
        self.size
    }

    fn map(
        &self,
        range: VirtAddrRange,
        flags: MappingFlags,
        _pt: &mut PageTable,
    ) -> LinuxResult<()> {
        debug!("Cow::map: {range:?} {flags:?}",);
        Ok(())
    }

    fn unmap(&self, range: VirtAddrRange, pt: &mut PageTable) -> LinuxResult<()> {
        debug!("Cow::unmap: {range:?}");
        for addr in pages_in(range, self.size)? {
            if let Ok((frame, _flags, _page_size)) = pt.unmap(addr) {
                if frame_table().dec_ref(frame) == 1 {
                    dealloc_frame(frame, self.size);
                }
            } else {
                // Deallocation is needn't if the page is not allocated.
            }
        }
        Ok(())
    }

    fn populate(
        &self,
        range: VirtAddrRange,
        flags: MappingFlags,
        access_flags: MappingFlags,
        pt: &mut PageTable,
    ) -> LinuxResult<(usize, Option<Box<dyn FnOnce(&mut AddrSpace)>>)> {
        let mut pages = 0;
        for addr in pages_in(range, self.size)? {
            match pt.query(addr) {
                Ok((paddr, page_flags, _)) => {
                    if access_flags.contains(MappingFlags::WRITE)
                        && !page_flags.contains(MappingFlags::WRITE)
                    {
                        self.handle_cow_fault(addr, paddr, flags, pt)?;
                        pages += 1;
                    }
                }
                // If the page is not mapped, try map it.
                Err(PagingError::NotMapped) => {
                    self.alloc_new_at(addr, flags, pt)?;
                    pages += 1;
                }
                Err(_) => return Err(LinuxError::EFAULT),
            }
        }
        Ok((pages, None))
    }

    fn clone_map(
        &self,
        range: VirtAddrRange,
        flags: MappingFlags,
        old_pt: &mut PageTable,
        new_pt: &mut PageTable,
        _new_aspace: &Arc<Mutex<AddrSpace>>,
    ) -> LinuxResult<Backend> {
        let cow_flags = flags - MappingFlags::WRITE;

        for vaddr in pages_in(range, self.size)? {
            // Copy data from old memory area to new memory area.
            match old_pt.query(vaddr) {
                Ok((paddr, _, page_size)) => {
                    // If the page is mapped in the old page table:
                    // - Update its permissions in the old page table using `flags`.
                    // - Map the same physical page into the new page table at the same
                    // virtual address, with the same page size and `flags`.
                    frame_table().inc_ref(paddr);

                    old_pt
                        .protect(vaddr, cow_flags)
                        .map_err(paging_to_linux_error)?;
                    new_pt
                        .map(vaddr, paddr, page_size, cow_flags)
                        .map_err(paging_to_linux_error)?;
                }
                // If the page is not mapped, skip it.
                Err(PagingError::NotMapped) => {}
                Err(_) => return Err(LinuxError::EFAULT),
            };
        }

        Ok(Backend::Cow(self.clone()))
    }
}

impl Backend {
    pub fn new_cow(
        start: VirtAddr,
        size: PageSize,
        file: Option<(FileBackend<RawMutex>, usize)>,
    ) -> Self {
        Self::Cow(CowBackend { start, size, file })
    }

    pub fn new_alloc(start: VirtAddr, size: PageSize) -> Self {
        Self::new_cow(start, size, None)
    }
}

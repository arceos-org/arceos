use axerrno::{LinuxError, LinuxResult};
use axhal::{
    mem::phys_to_virt,
    paging::{MappingFlags, PageSize, PageTable, PagingError},
};
use memory_addr::{PhysAddr, VirtAddr, VirtAddrRange};

use crate::{
    Backend,
    backend::{BackendOps, alloc_frame, dealloc_frame, pages_in, paging_to_linux_error},
    page_info::frame_table,
};

/// Allocation mapping backend.
///
/// If `populate` is `true`, all physical frames are allocated when the
/// mapping is created, and no page faults are triggered during the memory
/// access. Otherwise, the physical frames are allocated on demand (by
/// handling page faults).
#[derive(Clone)]
pub struct AllocBackend {
    populate: bool,
    size: PageSize,
}

impl AllocBackend {
    fn alloc_new_at(
        &self,
        vaddr: VirtAddr,
        flags: MappingFlags,
        pt: &mut PageTable,
    ) -> LinuxResult<()> {
        // Allocate a physical frame lazily and map it to the fault address.
        // `vaddr` does not need to be aligned. It will be automatically aligned
        // during `pt.map` regardless of the page size.
        let frame = alloc_frame(true, self.size)?;
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
                let new_frame = alloc_frame(false, self.size)?;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        phys_to_virt(paddr).as_ptr(),
                        phys_to_virt(new_frame).as_mut_ptr(),
                        self.size as _,
                    )
                };

                dealloc_frame(paddr, self.size);

                pt.remap(vaddr, new_frame, flags)
                    .map_err(paging_to_linux_error)?;
            }
        }

        Ok(())
    }
}

impl BackendOps for AllocBackend {
    fn page_size(&self) -> PageSize {
        self.size
    }

    fn map(
        &self,
        range: VirtAddrRange,
        flags: MappingFlags,
        pt: &mut PageTable,
    ) -> LinuxResult<()> {
        debug!(
            "Alloc::map: {range:?} {flags:?} (populate={})",
            self.populate
        );
        if !self.populate {
            return Ok(());
        }
        for addr in pages_in(range, self.size)? {
            let frame = alloc_frame(true, self.size)?;
            pt.map(addr, frame, self.size, flags)
                .map_err(paging_to_linux_error)?;
        }
        Ok(())
    }

    fn unmap(&self, range: VirtAddrRange, pt: &mut PageTable) -> LinuxResult<()> {
        debug!("unmap_alloc: {range:?}",);
        for addr in pages_in(range, self.size)? {
            if let Ok((frame, _page_size)) = pt.unmap(addr) {
                dealloc_frame(frame, self.size);
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
    ) -> LinuxResult<usize> {
        let mut pages = 0;
        for addr in pages_in(range, self.size)? {
            match pt.query(addr) {
                Ok((paddr, page_flags, _)) => {
                    if access_flags.contains(MappingFlags::WRITE)
                        && !page_flags.contains(MappingFlags::WRITE)
                    {
                        // If the page is mapped but not writable, we need to
                        // handle the COW fault.
                        self.handle_cow_fault(addr, paddr, flags, pt)?;
                        pages += 1;
                    }
                }
                // If the page is not mapped, try map it.
                Err(PagingError::NotMapped) => {
                    assert!(
                        !self.populate,
                        "populated backend should not have unpopulated pages"
                    );
                    self.alloc_new_at(addr, flags, pt)?;
                    pages += 1;
                }
                Err(_) => return Err(LinuxError::EFAULT),
            }
        }
        Ok(pages)
    }

    fn clone_map(
        &self,
        range: VirtAddrRange,
        flags: MappingFlags,
        old_pt: &mut PageTable,
        new_pt: &mut PageTable,
    ) -> LinuxResult<Backend> {
        let cow_flags = flags - MappingFlags::WRITE;

        // Forcing `populate = false` is to prevent the subsequent
        // `new_aspace.areas.map` from mapping page table entries for the
        // virtual addresses.
        let new_backend = Backend::new_alloc(false, self.size);

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

        Ok(new_backend)
    }
}

impl Backend {
    pub fn new_alloc(populate: bool, size: PageSize) -> Self {
        Self::Alloc(AllocBackend { populate, size })
    }
}

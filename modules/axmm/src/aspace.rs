use alloc::sync::Arc;
use core::{fmt, ops::DerefMut};

use axerrno::{AxError, AxResult, ax_bail};
use axhal::{
    mem::phys_to_virt,
    paging::{MappingFlags, PageTable},
    trap::PageFaultFlags,
};
use axsync::Mutex;
use memory_addr::{
    MemoryAddr, PAGE_SIZE_4K, PageIter4K, PhysAddr, VirtAddr, VirtAddrRange, is_aligned_4k,
};
use memory_set::{MemoryArea, MemorySet};

use crate::backend::{Backend, BackendOps};

/// The virtual memory address space.
pub struct AddrSpace {
    va_range: VirtAddrRange,
    areas: MemorySet<Backend>,
    pt: PageTable,
}

impl AddrSpace {
    /// Returns the address space base.
    pub const fn base(&self) -> VirtAddr {
        self.va_range.start
    }

    /// Returns the address space end.
    pub const fn end(&self) -> VirtAddr {
        self.va_range.end
    }

    /// Returns the address space size.
    pub fn size(&self) -> usize {
        self.va_range.size()
    }

    /// Returns the reference to the inner page table.
    pub const fn page_table(&self) -> &PageTable {
        &self.pt
    }

    /// Returns a mutable reference to the inner page table.
    pub const fn page_table_mut(&mut self) -> &mut PageTable {
        &mut self.pt
    }

    /// Returns the root physical address of the inner page table.
    pub const fn page_table_root(&self) -> PhysAddr {
        self.pt.root_paddr()
    }

    /// Checks if the address space contains the given address range.
    pub fn contains_range(&self, start: VirtAddr, size: usize) -> bool {
        self.va_range.contains(start) && (self.va_range.end - start) >= size
    }

    /// Creates a new empty address space.
    pub fn new_empty(base: VirtAddr, size: usize) -> AxResult<Self> {
        Ok(Self {
            va_range: VirtAddrRange::from_start_size(base, size),
            areas: MemorySet::new(),
            pt: PageTable::try_new().map_err(|_| AxError::NoMemory)?,
        })
    }

    /// Copies page table mappings from another address space.
    ///
    /// It copies the page table entries only rather than the memory regions,
    /// usually used to copy a portion of the kernel space mapping to the
    /// user space.
    ///
    /// Returns an error if the two address spaces overlap.
    #[cfg(feature = "copy")]
    pub fn copy_mappings_from(&mut self, other: &AddrSpace) -> AxResult {
        self.pt
            .modify()
            .copy_from(&other.pt, other.base(), other.size());
        Ok(())
    }

    fn validate_region(&self, start: VirtAddr, size: usize) -> AxResult {
        if !self.contains_range(start, size) {
            ax_bail!(NoMemory, "address out of range");
        }
        if !start.is_aligned_4k() || !is_aligned_4k(size) {
            ax_bail!(InvalidInput, "address is not aligned");
        }
        Ok(())
    }

    /// Finds a free area that can accommodate the given size.
    ///
    /// The search starts from the given hint address, and the area should be
    /// within the given limit range.
    ///
    /// Returns the start address of the free area. Returns None if no such area
    /// is found.
    pub fn find_free_area(
        &self,
        hint: VirtAddr,
        size: usize,
        limit: VirtAddrRange,
    ) -> Option<VirtAddr> {
        self.areas.find_free_area(hint, size, limit, PAGE_SIZE_4K)
    }

    pub fn find_area(&self, vaddr: VirtAddr) -> Option<&MemoryArea<Backend>> {
        self.areas.find(vaddr)
    }

    /// Add a new linear mapping.
    ///
    /// See [`Backend`] for more details about the mapping backends.
    ///
    /// The `flags` parameter indicates the mapping permissions and attributes.
    ///
    /// Returns an error if the address range is out of the address space or not
    /// aligned.
    pub fn map_linear(
        &mut self,
        start_vaddr: VirtAddr,
        start_paddr: PhysAddr,
        size: usize,
        flags: MappingFlags,
    ) -> AxResult {
        self.validate_region(start_vaddr, size)?;

        if !start_paddr.is_aligned_4k() {
            ax_bail!(InvalidInput, "address is not aligned");
        }

        let offset = start_vaddr.as_usize() as isize - start_paddr.as_usize() as isize;
        let area = MemoryArea::new(start_vaddr, size, flags, Backend::new_linear(offset));
        self.areas.map(area, &mut self.pt, false)?;
        Ok(())
    }

    pub fn map(
        &mut self,
        start: VirtAddr,
        size: usize,
        flags: MappingFlags,
        populate: bool,
        backend: Backend,
    ) -> AxResult {
        self.validate_region(start, size)?;

        let area = MemoryArea::new(start, size, flags, backend);
        self.areas.map(area, &mut self.pt, false)?;
        if populate {
            self.populate_area(start, size, flags)?;
        }
        Ok(())
    }

    /// Populates the area with physical frames, returning false if the area
    /// contains unmapped area.
    pub fn populate_area(
        &mut self,
        mut start: VirtAddr,
        size: usize,
        access_flags: MappingFlags,
    ) -> AxResult {
        self.validate_region(start, size)?;
        let end = start + size;

        let mut modify = self.pt.modify();
        while let Some(area) = self.areas.find(start) {
            let range = VirtAddrRange::new(start, area.end().min(end));
            area.backend()
                .populate(range, area.flags(), access_flags, &mut modify)?;
            start = area.end();
            assert!(start.is_aligned_4k());
            if start >= end {
                break;
            }
        }

        if start < end {
            // If the area is not fully mapped, we return ENOMEM.
            ax_bail!(NoMemory);
        }

        Ok(())
    }

    /// Removes mappings within the specified virtual address range.
    ///
    /// Returns an error if the address range is out of the address space or not
    /// aligned.
    pub fn unmap(&mut self, start: VirtAddr, size: usize) -> AxResult {
        self.validate_region(start, size)?;

        self.areas.unmap(start, size, &mut self.pt)?;
        Ok(())
    }

    /// To process data in this area with the given function.
    ///
    /// Now it supports reading and writing data in the given interval.
    fn process_area_data<F>(&self, start: VirtAddr, size: usize, mut f: F) -> AxResult
    where
        F: FnMut(VirtAddr, usize, usize),
    {
        if !self.contains_range(start, size) {
            ax_bail!(InvalidInput, "address out of range");
        }
        let mut cnt = 0;
        // If start is aligned to 4K, start_align_down will be equal to start_align_up.
        let end_align_up = (start + size).align_up_4k();
        for vaddr in PageIter4K::new(start.align_down_4k(), end_align_up)
            .expect("Failed to create page iterator")
        {
            let (mut paddr, ..) = self.pt.query(vaddr).map_err(|_| AxError::BadAddress)?;

            let mut copy_size = (size - cnt).min(PAGE_SIZE_4K);

            if copy_size == 0 {
                break;
            }
            if vaddr == start.align_down_4k() && start.align_offset_4k() != 0 {
                let align_offset = start.align_offset_4k();
                copy_size = copy_size.min(PAGE_SIZE_4K - align_offset);
                paddr += align_offset;
            }
            f(phys_to_virt(paddr), cnt, copy_size);
            cnt += copy_size;
        }
        Ok(())
    }

    /// To read data from the address space.
    ///
    /// # Arguments
    ///
    /// * `start` - The start virtual address to read.
    /// * `buf` - The buffer to store the data.
    pub fn read(&self, start: VirtAddr, buf: &mut [u8]) -> AxResult {
        self.process_area_data(start, buf.len(), |src, offset, read_size| unsafe {
            core::ptr::copy_nonoverlapping(src.as_ptr(), buf.as_mut_ptr().add(offset), read_size);
        })
    }

    /// To write data to the address space.
    ///
    /// # Arguments
    ///
    /// * `start_vaddr` - The start virtual address to write.
    /// * `buf` - The buffer to write to the address space.
    pub fn write(&self, start: VirtAddr, buf: &[u8]) -> AxResult {
        self.process_area_data(start, buf.len(), |dst, offset, write_size| unsafe {
            core::ptr::copy_nonoverlapping(buf.as_ptr().add(offset), dst.as_mut_ptr(), write_size);
        })
    }

    /// Updates mapping within the specified virtual address range.
    ///
    /// Returns an error if the address range is out of the address space or not
    /// aligned.
    pub fn protect(&mut self, start: VirtAddr, size: usize, flags: MappingFlags) -> AxResult {
        self.validate_region(start, size)?;

        self.areas
            .protect(start, size, |_| Some(flags), &mut self.pt)?;

        Ok(())
    }

    /// Removes all mappings in the address space.
    pub fn clear(&mut self) {
        self.areas.clear(&mut self.pt).unwrap();
    }

    /// Checks whether an access to the specified memory region is valid.
    ///
    /// Returns `true` if the memory region given by `range` is all mapped and
    /// has proper permission flags (i.e. containing `access_flags`).
    pub fn can_access_range(
        &self,
        start: VirtAddr,
        size: usize,
        access_flags: MappingFlags,
    ) -> bool {
        let Some(mut range) = VirtAddrRange::try_from_start_size(start, size) else {
            return false;
        };
        for area in self.areas.iter() {
            if area.end() <= range.start {
                continue;
            }
            if area.start() > range.start {
                return false;
            }

            // This area overlaps with the memory region
            if !area.flags().contains(access_flags) {
                return false;
            }

            range.start = area.end();
            if range.is_empty() {
                return true;
            }
        }

        false
    }

    /// Handles a page fault at the given address.
    ///
    /// `access_flags` indicates the access type that caused the page fault.
    ///
    /// Returns `true` if the page fault is handled successfully (not a real
    /// fault).
    pub fn handle_page_fault(&mut self, vaddr: VirtAddr, access_flags: PageFaultFlags) -> bool {
        if !self.va_range.contains(vaddr) {
            return false;
        }
        if let Some(area) = self.areas.find(vaddr) {
            let flags = area.flags();
            if flags.contains(access_flags) {
                let page_size = area.backend().page_size();
                let populate_result = area.backend().populate(
                    VirtAddrRange::from_start_size(vaddr.align_down(page_size), page_size as _),
                    flags,
                    access_flags,
                    &mut self.pt.modify(),
                );
                return match populate_result {
                    Ok((n, callback)) => {
                        if let Some(cb) = callback {
                            cb(self);
                        }
                        if n == 0 {
                            warn!("No pages populated for {vaddr:?} ({flags:?})");
                            false
                        } else {
                            true
                        }
                    }
                    Err(err) => {
                        warn!("Failed to populate pages for {vaddr:?} ({flags:?}): {err}");
                        false
                    }
                };
            }
        }
        false
    }

    /// Attempts to clone the current address space into a new one.
    ///
    /// This method creates a new empty address space with the same base and
    /// size, then iterates over all memory areas in the original address
    /// space to copy or share their mappings into the new one.
    pub fn try_clone(&mut self) -> AxResult<Arc<Mutex<Self>>> {
        let new_aspace = Arc::new(Mutex::new(Self::new_empty(self.base(), self.size())?));
        let new_aspace_clone = new_aspace.clone();

        let mut guard = new_aspace.lock();

        let mut self_modify = self.pt.modify();
        for area in self.areas.iter() {
            let new_backend = area.backend().clone_map(
                area.va_range(),
                area.flags(),
                &mut self_modify,
                &mut guard.pt.modify(),
                &new_aspace_clone,
            )?;

            let new_area = MemoryArea::new(area.start(), area.size(), area.flags(), new_backend);
            let aspace = guard.deref_mut();
            aspace.areas.map(new_area, &mut aspace.pt, false)?;
        }
        drop(guard);

        Ok(new_aspace)
    }
}

impl fmt::Debug for AddrSpace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AddrSpace")
            .field("va_range", &self.va_range)
            .field("page_table_root", &self.pt.root_paddr())
            .field("areas", &self.areas)
            .finish()
    }
}

impl Drop for AddrSpace {
    fn drop(&mut self) {
        self.clear();
    }
}

use core::fmt;

use axerrno::{ax_err, AxError, AxResult};
use axhal::paging::{MappingFlags, PageTable};
use memory_addr::{is_aligned_4k, MemoryAddr, PhysAddr, VirtAddr, VirtAddrRange};
use memory_set::{MemoryArea, MemorySet};

use crate::backend::Backend;
use crate::mapping_err_to_ax_err;

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

    /// Returns the root physical address of the inner page table.
    pub const fn page_table_root(&self) -> PhysAddr {
        self.pt.root_paddr()
    }

    /// Checks if the address space contains the given address range.
    pub fn contains_range(&self, start: VirtAddr, size: usize) -> bool {
        self.va_range
            .contains_range(VirtAddrRange::from_start_size(start, size))
    }

    /// Creates a new empty address space.
    pub(crate) fn new_empty(base: VirtAddr, size: usize) -> AxResult<Self> {
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
    pub fn copy_mappings_from(&mut self, other: &AddrSpace) -> AxResult {
        if self.va_range.overlaps(other.va_range) {
            return ax_err!(InvalidInput, "address space overlap");
        }
        self.pt.copy_from(&other.pt, other.base(), other.size());
        Ok(())
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
        if !self.contains_range(start_vaddr, size) {
            return ax_err!(InvalidInput, "address out of range");
        }
        if !start_vaddr.is_aligned_4k() || !start_paddr.is_aligned_4k() || !is_aligned_4k(size) {
            return ax_err!(InvalidInput, "address not aligned");
        }

        let offset = start_vaddr.as_usize() - start_paddr.as_usize();
        let area = MemoryArea::new(start_vaddr, size, flags, Backend::new_linear(offset));
        self.areas
            .map(area, &mut self.pt, false)
            .map_err(mapping_err_to_ax_err)?;
        Ok(())
    }

    /// Add a new allocation mapping.
    ///
    /// See [`Backend`] for more details about the mapping backends.
    ///
    /// The `flags` parameter indicates the mapping permissions and attributes.
    ///
    /// Returns an error if the address range is out of the address space or not
    /// aligned.
    pub fn map_alloc(
        &mut self,
        start: VirtAddr,
        size: usize,
        flags: MappingFlags,
        populate: bool,
    ) -> AxResult {
        if !self.contains_range(start, size) {
            return ax_err!(InvalidInput, "address out of range");
        }
        if !start.is_aligned_4k() || !is_aligned_4k(size) {
            return ax_err!(InvalidInput, "address not aligned");
        }

        let area = MemoryArea::new(start, size, flags, Backend::new_alloc(populate));
        self.areas
            .map(area, &mut self.pt, false)
            .map_err(mapping_err_to_ax_err)?;
        Ok(())
    }

    /// Removes mappings within the specified virtual address range.
    ///
    /// Returns an error if the address range is out of the address space or not
    /// aligned.
    pub fn unmap(&mut self, start: VirtAddr, size: usize) -> AxResult {
        if !self.contains_range(start, size) {
            return ax_err!(InvalidInput, "address out of range");
        }
        if !start.is_aligned_4k() || !is_aligned_4k(size) {
            return ax_err!(InvalidInput, "address not aligned");
        }

        self.areas
            .unmap(start, size, &mut self.pt)
            .map_err(mapping_err_to_ax_err)?;
        Ok(())
    }

    /// Updates mapping within the specified virtual address range.
    ///
    /// Returns an error if the address range is out of the address space or not
    /// aligned.
    pub fn protect(&mut self, start: VirtAddr, size: usize, flags: MappingFlags) -> AxResult {
        if !self.contains_range(start, size) {
            return ax_err!(InvalidInput, "address out of range");
        }
        if !start.is_aligned_4k() || !is_aligned_4k(size) {
            return ax_err!(InvalidInput, "address not aligned");
        }

        // TODO
        self.pt
            .protect_region(start, size, flags, true)
            .map_err(|_| AxError::BadState)?
            .ignore();
        Ok(())
    }

    /// Removes all mappings in the address space.
    pub fn clear(&mut self) {
        self.areas.clear(&mut self.pt).unwrap();
    }

    /// Handles a page fault at the given address.
    ///
    /// `access_flags` indicates the access type that caused the page fault.
    ///
    /// Returns `true` if the page fault is handled successfully (not a real
    /// fault).
    pub fn handle_page_fault(&mut self, vaddr: VirtAddr, access_flags: MappingFlags) -> bool {
        if !self.va_range.contains(vaddr) {
            return false;
        }
        if let Some(area) = self.areas.find(vaddr) {
            let orig_flags = area.flags();
            if orig_flags.contains(access_flags) {
                return area
                    .backend()
                    .handle_page_fault(vaddr, orig_flags, &mut self.pt);
            }
        }
        false
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

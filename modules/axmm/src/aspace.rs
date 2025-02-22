use core::fmt;

use alloc::vec;
use axerrno::{AxError, AxResult, ax_err};
use axhal::mem::phys_to_virt;
use axhal::paging::{MappingFlags, PageTable};
use memory_addr::{
    MemoryAddr, PAGE_SIZE_4K, PageIter4K, PhysAddr, VirtAddr, VirtAddrRange, is_aligned_4k,
};
use memory_set::{MemoryArea, MemorySet};

use crate::backend::Backend;
use crate::{KERNEL_ASPACE, mapping_err_to_ax_err};

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

    /// Finds a free area that can accommodate the given size.
    ///
    /// The search starts from the given hint address, and the area should be within the given limit range.
    ///
    /// Returns the start address of the free area. Returns None if no such area is found.
    pub fn find_free_area(
        &self,
        hint: VirtAddr,
        size: usize,
        limit: VirtAddrRange,
    ) -> Option<VirtAddr> {
        self.areas.find_free_area(hint, size, limit)
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

    /// Add a new zero-initialized allocation mapping.
    pub fn alloc_for_lazy(&mut self, start: VirtAddr, size: usize) -> AxResult {
        let end = (start + size).align_up_4k();
        let mut start = start.align_down_4k();
        let size = end - start;
        if !self.contains_range(start, size) {
            return ax_err!(InvalidInput, "address out of range");
        }
        while let Some(area) = self.areas.find(start) {
            let area_backend = area.backend();
            if let Backend::Alloc { populate } = area_backend {
                if !*populate {
                    let count = (area.end().min(end) - start).align_up_4k() / PAGE_SIZE_4K;
                    for i in 0..count {
                        let addr = start + i * PAGE_SIZE_4K;
                        area_backend.handle_page_fault_alloc(
                            addr,
                            area.flags(),
                            &mut self.pt,
                            *populate,
                        );
                    }
                }
            }
            start = area.end();
            assert!(start.is_aligned_4k());
        }
        if start < end {
            ax_err!(InvalidInput, "address out of range")?;
        }
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

    /// To remove user area mappings from address space.
    pub fn unmap_user_areas(&mut self) -> AxResult {
        for area in self.areas.iter() {
            assert!(area.start().is_aligned_4k());
            assert!(area.size() % PAGE_SIZE_4K == 0);
            assert!(area.flags().contains(MappingFlags::USER));
            assert!(
                self.va_range
                    .contains_range(VirtAddrRange::from_start_size(area.start(), area.size())),
                "MemorySet contains out-of-va-range area"
            );
        }
        self.areas.clear(&mut self.pt).unwrap();
        Ok(())
    }

    /// To process data in this area with the given function.
    ///
    /// Now it supports reading and writing data in the given interval.
    ///
    /// # Arguments
    /// - `start`: The start virtual address to process.
    /// - `size`: The size of the data to process.
    /// - `f`: The function to process the data, whose arguments are the start virtual address,
    ///   the offset and the size of the data.
    ///
    /// # Notes
    ///   The caller must ensure that the permission of the operation is allowed.
    fn process_area_data<F>(&self, start: VirtAddr, size: usize, f: F) -> AxResult
    where
        F: FnMut(VirtAddr, usize, usize),
    {
        Self::process_area_data_with_page_table(&self.pt, &self.va_range, start, size, f)
    }

    fn process_area_data_with_page_table<F>(
        pt: &PageTable,
        va_range: &VirtAddrRange,
        start: VirtAddr,
        size: usize,
        mut f: F,
    ) -> AxResult
    where
        F: FnMut(VirtAddr, usize, usize),
    {
        if !va_range.contains_range(VirtAddrRange::from_start_size(start, size)) {
            return ax_err!(InvalidInput, "address out of range");
        }
        let mut cnt = 0;
        // If start is aligned to 4K, start_align_down will be equal to start_align_up.
        let end_align_up = (start + size).align_up_4k();
        for vaddr in PageIter4K::new(start.align_down_4k(), end_align_up)
            .expect("Failed to create page iterator")
        {
            let (mut paddr, _, _) = pt.query(vaddr).map_err(|_| AxError::BadAddress)?;

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

    /// 克隆 AddrSpace。这将创建一个新的页表，并将旧页表中的所有区域（包括内核区域）映射到新的页表中，但仅将用户区域的映射到新的 MemorySet 中。
    ///
    /// 如果发生错误，新创建的 MemorySet 将被丢弃并返回错误。
    pub fn clone_or_err(&mut self) -> AxResult<Self> {
        // 由于要克隆的这个地址空间可能是用户空间，而用户空间在一开始创建时不会在MemorySet中管理内核区域，而是直接把相关的页表项复制到了新页表中，所以在MemorySet中没有内核区域，需要另外处理。
        let mut new_pt = PageTable::try_new().map_err(|_| AxError::NoMemory)?;
        // 如果不是 ARMv8 架构，将内核部分复制到用户页表中。
        if !cfg!(target_arch = "aarch64") && !cfg!(target_arch = "loongarch64") {
            // ARMv8 使用一个单独的页表 (TTBR0_EL1) 用于用户空间，不需要将内核部分复制到用户页表中。
            let kernel_aspace = KERNEL_ASPACE.lock();
            new_pt.copy_from(
                &kernel_aspace.pt,
                kernel_aspace.base(),
                kernel_aspace.size(),
            );
        }

        // 创建一个新的 MemorySet 并将原始区域映射到新的页表中。
        let mut new_areas = MemorySet::new();
        let mut buf = vec![0u8; PAGE_SIZE_4K];
        for area in self.areas.iter() {
            let new_area = MemoryArea::new(
                area.start(),
                area.size(),
                area.flags(),
                area.backend().clone(),
            );
            new_areas
                .map(new_area, &mut new_pt, false)
                .map_err(mapping_err_to_ax_err)?;
            // 将原区域的数据复制到新区域中。
            buf.resize(buf.capacity().max(area.size()), 0);
            self.read(area.start(), &mut buf).inspect_err(|_| {
                new_areas.clear(&mut new_pt).unwrap();
            })?;
            Self::process_area_data_with_page_table(
                &new_pt,
                &self.va_range,
                area.start(),
                area.size(),
                |dst, offset, write_size| unsafe {
                    core::ptr::copy_nonoverlapping(
                        buf.as_ptr().add(offset),
                        dst.as_mut_ptr(),
                        write_size,
                    );
                },
            )?;
        }
        Ok(Self {
            va_range: self.va_range,
            areas: new_areas,
            pt: new_pt,
        })
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

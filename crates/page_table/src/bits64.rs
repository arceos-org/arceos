extern crate alloc;

use alloc::{vec, vec::Vec};
use core::marker::PhantomData;

use memory_addr::{PhysAddr, VirtAddr, PAGE_SIZE_4K};

use crate::{GenericPTE, PagingIf, PagingMetaData};
use crate::{MappingFlags, PageSize, PagingError, PagingResult};

const ENTRY_COUNT: usize = 512;

const fn p4_index(vaddr: VirtAddr) -> usize {
    (vaddr.as_usize() >> (12 + 27)) & (ENTRY_COUNT - 1)
}

const fn p3_index(vaddr: VirtAddr) -> usize {
    (vaddr.as_usize() >> (12 + 18)) & (ENTRY_COUNT - 1)
}

const fn p2_index(vaddr: VirtAddr) -> usize {
    (vaddr.as_usize() >> (12 + 9)) & (ENTRY_COUNT - 1)
}

const fn p1_index(vaddr: VirtAddr) -> usize {
    (vaddr.as_usize() >> 12) & (ENTRY_COUNT - 1)
}

pub struct PageTable64<M: PagingMetaData, PTE: GenericPTE, IF: PagingIf> {
    root_paddr: PhysAddr,
    intrm_tables: Vec<PhysAddr>,
    _phantom: PhantomData<(M, PTE, IF)>,
}

impl<M: PagingMetaData, PTE: GenericPTE, IF: PagingIf> PageTable64<M, PTE, IF> {
    pub fn try_new() -> PagingResult<Self> {
        let root_paddr = Self::alloc_table()?;
        Ok(Self {
            root_paddr,
            intrm_tables: vec![root_paddr],
            _phantom: PhantomData,
        })
    }

    pub const fn root_paddr(&self) -> PhysAddr {
        self.root_paddr
    }

    pub fn map(
        &mut self,
        vaddr: VirtAddr,
        target: PhysAddr,
        page_size: PageSize,
        flags: MappingFlags,
    ) -> PagingResult {
        let entry = self.get_entry_mut_or_create(vaddr, page_size)?;
        if !entry.is_unused() {
            return Err(PagingError::AlreadyMapped);
        }
        *entry = GenericPTE::new_page(target.align_down(page_size), flags, page_size.is_huge());
        Ok(())
    }

    pub fn map_fault(&mut self, vaddr: VirtAddr, page_size: PageSize) -> PagingResult {
        let entry = self.get_entry_mut_or_create(vaddr, page_size)?;
        if !entry.is_unused() {
            return Err(PagingError::AlreadyMapped);
        }
        *entry = GenericPTE::new_fault_page(page_size.is_huge());
        Ok(())
    }

    /// Same as `PageTable64::map()`. This function will error if entry doesn't exist. Should be
    /// used to edit PTE in page fault handler.
    pub fn map_overwrite(
        &mut self,
        vaddr: VirtAddr,
        target: PhysAddr,
        page_size: PageSize,
        flags: MappingFlags,
    ) -> PagingResult {
        let entry = self.get_entry_mut_or_create(vaddr, page_size)?;
        if entry.is_unused() {
            return Err(PagingError::AlreadyMapped);
        }
        *entry = GenericPTE::new_page(target.align_down(page_size), flags, page_size.is_huge());
        Ok(())
    }

    pub fn unmap(&mut self, vaddr: VirtAddr) -> PagingResult<(PhysAddr, PageSize)> {
        let (entry, size) = self.get_entry_mut(vaddr)?;
        if entry.is_unused() {
            return Err(PagingError::NotMapped);
        }
        let paddr = entry.paddr();
        entry.clear();
        Ok((paddr, size))
    }

    pub fn query(&self, vaddr: VirtAddr) -> PagingResult<(PhysAddr, MappingFlags, PageSize)> {
        let (entry, size) = self.get_entry_mut(vaddr)?;
        if entry.is_unused() {
            return Err(PagingError::NotMapped);
        }
        let off = vaddr.align_offset(size);
        Ok((entry.paddr() + off, entry.flags(), size))
    }

    pub fn lookup(
        paddr: PhysAddr,
        vaddr: VirtAddr,
    ) -> PagingResult<(PhysAddr, MappingFlags, PageSize)> {
        let (entry, size) = Self::lookup_entry_mut(paddr, vaddr)?;
        if entry.is_unused() {
            return Err(PagingError::NotMapped);
        }
        let off = vaddr.align_offset(size);
        Ok((entry.paddr() + off, entry.flags(), size))
    }

    pub fn map_region(
        &mut self,
        vaddr: VirtAddr,
        paddr: PhysAddr,
        size: usize,
        flags: MappingFlags,
        allow_huge: bool,
    ) -> PagingResult {
        if !vaddr.is_aligned(PageSize::Size4K)
            || !paddr.is_aligned(PageSize::Size4K)
            || !memory_addr::is_aligned(size, PageSize::Size4K as usize)
        {
            return Err(PagingError::NotAligned);
        }
        trace!(
            "map_region({:#x}): [{:#x}, {:#x}) -> [{:#x}, {:#x}) ({:#?})",
            self.root_paddr(),
            vaddr,
            vaddr + size,
            paddr,
            paddr + size,
            flags,
        );
        let mut vaddr = vaddr;
        let mut paddr = paddr;
        let mut size = size;
        while size > 0 {
            let page_size = if allow_huge {
                if vaddr.is_aligned(PageSize::Size1G)
                    && paddr.is_aligned(PageSize::Size1G)
                    && size >= PageSize::Size1G as usize
                {
                    PageSize::Size1G
                } else if vaddr.is_aligned(PageSize::Size2M)
                    && paddr.is_aligned(PageSize::Size2M)
                    && size >= PageSize::Size2M as usize
                {
                    PageSize::Size2M
                } else {
                    PageSize::Size4K
                }
            } else {
                PageSize::Size4K
            };
            self.map(vaddr, paddr, page_size, flags).inspect_err(|e| {
                error!(
                    "failed to map page: {:#x?}({:?}) -> {:#x?}, {:?}",
                    vaddr, page_size, paddr, e
                )
            })?;
            vaddr += page_size as usize;
            paddr += page_size as usize;
            size -= page_size as usize;
        }
        Ok(())
    }

    /// TODO: huge page
    pub fn map_fault_region(&mut self, mut vaddr: VirtAddr, mut size: usize) -> PagingResult {
        if !vaddr.is_aligned(PageSize::Size4K)
            || !memory_addr::is_aligned(size, PageSize::Size4K as usize)
        {
            return Err(PagingError::NotAligned);
        }
        trace!(
            "map_fulat_region({:#x}): [{:#x}, {:#x})",
            self.root_paddr(),
            vaddr,
            vaddr + size,
        );

        while size > 0 {
            self.map_fault(vaddr, PageSize::Size4K).inspect_err(|e| {
                error!(
                    "failed to map fault page: {:#x?}({:?}), {:?}",
                    vaddr,
                    PageSize::Size4K,
                    e
                )
            })?;
            vaddr += PageSize::Size4K as usize;
            size -= PageSize::Size4K as usize;
        }

        Ok(())
    }

    pub fn unmap_region(&mut self, vaddr: VirtAddr, size: usize) -> PagingResult {
        trace!(
            "unmap_region({:#x}) [{:#x}, {:#x})",
            self.root_paddr(),
            vaddr,
            vaddr + size,
        );
        let mut vaddr = vaddr;
        let mut size = size;
        while size > 0 {
            let (_, page_size) = self
                .unmap(vaddr)
                .inspect_err(|e| error!("failed to unmap page: {:#x?}, {:?}", vaddr, e))?;
            assert!(vaddr.is_aligned(page_size));
            assert!(page_size as usize <= size);
            vaddr += page_size as usize;
            size -= page_size as usize;
        }
        Ok(())
    }

    /// Update the mapping flags of an entry. Return the page size on success.
    pub fn update(&mut self, vaddr: VirtAddr, flags: MappingFlags) -> PagingResult<PageSize> {
        let (pte, page_size) = self.get_entry_mut(vaddr)?;

        *pte = GenericPTE::new_page(pte.paddr(), flags, pte.is_huge());

        Ok(page_size)
    }

    pub fn update_region(
        &mut self,
        mut vaddr: VirtAddr,
        size: usize,
        flags: MappingFlags,
    ) -> PagingResult {
        let end = vaddr + size;
        while vaddr < end {
            let page_size = self.update(vaddr, flags)?;
            vaddr += page_size as usize;
        }
        Ok(())
    }

    pub fn walk<F>(&self, limit: usize, func: &F) -> PagingResult
    where
        F: Fn(usize, usize, VirtAddr, &PTE),
    {
        self.walk_recursive(
            self.table_of(self.root_paddr()),
            0,
            VirtAddr::from(0),
            limit,
            func,
        )
    }
}

// Private implements.
impl<M: PagingMetaData, PTE: GenericPTE, IF: PagingIf> PageTable64<M, PTE, IF> {
    fn alloc_table() -> PagingResult<PhysAddr> {
        if let Some(paddr) = IF::alloc_frame() {
            let ptr = IF::phys_to_virt(paddr).as_mut_ptr();
            unsafe { core::ptr::write_bytes(ptr, 0, PAGE_SIZE_4K) };
            Ok(paddr)
        } else {
            Err(PagingError::NoMemory)
        }
    }

    fn table_of<'a>(&self, paddr: PhysAddr) -> &'a [PTE] {
        let ptr = IF::phys_to_virt(paddr).as_ptr() as _;
        unsafe { core::slice::from_raw_parts(ptr, ENTRY_COUNT) }
    }

    fn table_of_mut<'a>(&self, paddr: PhysAddr) -> &'a mut [PTE] {
        let ptr = IF::phys_to_virt(paddr).as_mut_ptr() as _;
        unsafe { core::slice::from_raw_parts_mut(ptr, ENTRY_COUNT) }
    }

    /// Get PTE slice for page table in `paddr`
    ///
    /// SAFETY: paddr must be a valid page table and the slice returned should live shorter than
    /// the page table.
    unsafe fn lookup_table_of_mut<'a>(paddr: PhysAddr) -> &'a mut [PTE] {
        let ptr = IF::phys_to_virt(paddr).as_mut_ptr() as _;
        core::slice::from_raw_parts_mut(ptr, ENTRY_COUNT)
    }

    fn next_table_mut<'a>(&self, entry: &PTE) -> PagingResult<&'a mut [PTE]> {
        if !entry.is_present() {
            Err(PagingError::NotMapped)
        } else if entry.is_huge() {
            Err(PagingError::MappedToHugePage)
        } else {
            Ok(self.table_of_mut(entry.paddr()))
        }
    }

    fn lookup_next_table_mut<'a>(entry: &PTE) -> PagingResult<&'a mut [PTE]> {
        if !entry.is_present() {
            Err(PagingError::NotMapped)
        } else if entry.is_huge() {
            Err(PagingError::MappedToHugePage)
        } else {
            unsafe { Ok(Self::lookup_table_of_mut(entry.paddr())) }
        }
    }

    fn next_table_mut_or_create<'a>(&mut self, entry: &mut PTE) -> PagingResult<&'a mut [PTE]> {
        if entry.is_unused() {
            let paddr = Self::alloc_table()?;
            self.intrm_tables.push(paddr);
            *entry = GenericPTE::new_table(paddr);
            Ok(self.table_of_mut(paddr))
        } else {
            self.next_table_mut(entry)
        }
    }

    /// Loop up the page table in `paddr` manually to query for PTE.
    ///
    /// SAFETY: `paddr` must be a valid page table and &PTE returned should live shorter than the
    /// page table itself.
    fn lookup_entry_mut<'a>(
        paddr: PhysAddr,
        vaddr: VirtAddr,
    ) -> PagingResult<(&'a mut PTE, PageSize)> {
        let p3 = if M::LEVELS == 3 {
            unsafe { Self::lookup_table_of_mut(paddr) }
        } else if M::LEVELS == 4 {
            let p4 = unsafe { Self::lookup_table_of_mut(paddr) };
            let p4e = &mut p4[p4_index(vaddr)];
            Self::lookup_next_table_mut(p4e)?
        } else {
            unreachable!()
        };

        let p3e = &mut p3[p3_index(vaddr)];
        if p3e.is_huge() {
            return Ok((p3e, PageSize::Size1G));
        }

        let p2 = Self::lookup_next_table_mut(p3e)?;
        let p2e = &mut p2[p2_index(vaddr)];
        if p2e.is_huge() {
            return Ok((p2e, PageSize::Size2M));
        }

        let p1 = Self::lookup_next_table_mut(p2e)?;
        let p1e = &mut p1[p1_index(vaddr)];
        Ok((p1e, PageSize::Size4K))
    }

    fn get_entry_mut(&self, vaddr: VirtAddr) -> PagingResult<(&mut PTE, PageSize)> {
        let p3 = if M::LEVELS == 3 {
            self.table_of_mut(self.root_paddr())
        } else if M::LEVELS == 4 {
            let p4 = self.table_of_mut(self.root_paddr());
            let p4e = &mut p4[p4_index(vaddr)];
            self.next_table_mut(p4e)?
        } else {
            unreachable!()
        };
        let p3e = &mut p3[p3_index(vaddr)];
        if p3e.is_huge() {
            return Ok((p3e, PageSize::Size1G));
        }

        let p2 = self.next_table_mut(p3e)?;
        let p2e = &mut p2[p2_index(vaddr)];
        if p2e.is_huge() {
            return Ok((p2e, PageSize::Size2M));
        }

        let p1 = self.next_table_mut(p2e)?;
        let p1e = &mut p1[p1_index(vaddr)];
        Ok((p1e, PageSize::Size4K))
    }

    fn get_entry_mut_or_create(
        &mut self,
        vaddr: VirtAddr,
        page_size: PageSize,
    ) -> PagingResult<&mut PTE> {
        let p3 = if M::LEVELS == 3 {
            self.table_of_mut(self.root_paddr())
        } else if M::LEVELS == 4 {
            let p4 = self.table_of_mut(self.root_paddr());
            let p4e = &mut p4[p4_index(vaddr)];
            self.next_table_mut_or_create(p4e)?
        } else {
            unreachable!()
        };
        let p3e = &mut p3[p3_index(vaddr)];
        if page_size == PageSize::Size1G {
            return Ok(p3e);
        }

        let p2 = self.next_table_mut_or_create(p3e)?;
        let p2e = &mut p2[p2_index(vaddr)];
        if page_size == PageSize::Size2M {
            return Ok(p2e);
        }

        let p1 = self.next_table_mut_or_create(p2e)?;
        let p1e = &mut p1[p1_index(vaddr)];
        Ok(p1e)
    }

    fn walk_recursive<F>(
        &self,
        table: &[PTE],
        level: usize,
        start_vaddr: VirtAddr,
        limit: usize,
        func: &F,
    ) -> PagingResult
    where
        F: Fn(usize, usize, VirtAddr, &PTE),
    {
        let mut n = 0;
        for (i, entry) in table.iter().enumerate() {
            let vaddr = start_vaddr + (i << (12 + (M::LEVELS - 1 - level) * 9));
            if entry.is_present() {
                func(level, i, vaddr, entry);
                if level < M::LEVELS - 1 && !entry.is_huge() {
                    let table_entry = self.next_table_mut(entry)?;
                    self.walk_recursive(table_entry, level + 1, vaddr, limit, func)?;
                }
                n += 1;
                if n >= limit {
                    break;
                }
            }
        }
        Ok(())
    }
}

impl<M: PagingMetaData, PTE: GenericPTE, IF: PagingIf> Drop for PageTable64<M, PTE, IF> {
    fn drop(&mut self) {
        for frame in &self.intrm_tables {
            IF::dealloc_frame(*frame);
        }
    }
}

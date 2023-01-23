use alloc::{vec, vec::Vec};
use core::marker::PhantomData;

use crate::{GenericPTE, MappingFlags, PagingError, PagingIf, PagingResult};
use crate::{Page, PageSize, PageTableLevels, PhysAddr, VirtAddr, PAGE_SIZE_4K};

const ENTRY_COUNT: usize = 512;

const fn p4_index(vaddr: VirtAddr) -> usize {
    (vaddr >> (12 + 27)) & (ENTRY_COUNT - 1)
}

const fn p3_index(vaddr: VirtAddr) -> usize {
    (vaddr >> (12 + 18)) & (ENTRY_COUNT - 1)
}

const fn p2_index(vaddr: VirtAddr) -> usize {
    (vaddr >> (12 + 9)) & (ENTRY_COUNT - 1)
}

const fn p1_index(vaddr: VirtAddr) -> usize {
    (vaddr >> 12) & (ENTRY_COUNT - 1)
}

pub struct PageTable64<L: PageTableLevels, PTE: GenericPTE, IF: PagingIf> {
    root_paddr: PhysAddr,
    intrm_tables: Vec<PhysAddr>,
    _phantom: PhantomData<(L, PTE, IF)>,
}

impl<L: PageTableLevels, PTE: GenericPTE, IF: PagingIf> PageTable64<L, PTE, IF> {
    pub fn new() -> PagingResult<Self> {
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

    pub fn map(&mut self, page: Page, target: PhysAddr, flags: MappingFlags) -> PagingResult {
        let entry = self.get_entry_mut_or_create(page)?;
        if !entry.is_unused() {
            return Err(PagingError::AlreadyMapped);
        }
        *entry = GenericPTE::new_page(page.size.align_down(target), flags, page.size.is_huge());
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
        let off = size.page_offset(vaddr.into());
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
        if !PageSize::Size4K.is_aligned(vaddr)
            || !PageSize::Size4K.is_aligned(paddr)
            || !PageSize::Size4K.is_aligned(size)
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
                if PageSize::Size1G.is_aligned(vaddr)
                    && PageSize::Size1G.is_aligned(paddr)
                    && size >= PageSize::Size1G as usize
                {
                    PageSize::Size1G
                } else if PageSize::Size2M.is_aligned(vaddr)
                    && PageSize::Size2M.is_aligned(paddr)
                    && size >= PageSize::Size2M as usize
                {
                    PageSize::Size2M
                } else {
                    PageSize::Size4K
                }
            } else {
                PageSize::Size4K
            };
            let page = Page::new_aligned(vaddr, page_size);
            self.map(page, paddr, flags).inspect_err(|e| {
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
            assert!(page_size.is_aligned(vaddr));
            assert!(page_size as usize <= size);
            vaddr += page_size as usize;
            size -= page_size as usize;
        }
        Ok(())
    }

    pub fn walk(&self, limit: usize, func: &impl Fn(usize, usize, VirtAddr, &PTE)) -> PagingResult {
        self.walk_recursive(self.table_of(self.root_paddr()), 0, 0, limit, func)
    }
}

// Private implements.
impl<L: PageTableLevels, PTE: GenericPTE, IF: PagingIf> PageTable64<L, PTE, IF> {
    fn alloc_table() -> PagingResult<PhysAddr> {
        if let Some(paddr) = IF::alloc_frame() {
            let ptr = IF::phys_to_virt(paddr) as *mut u8;
            unsafe { core::ptr::write_bytes(ptr, 0, PAGE_SIZE_4K) };
            Ok(paddr)
        } else {
            Err(PagingError::NoMemory)
        }
    }

    fn table_of<'a>(&self, paddr: PhysAddr) -> &'a [PTE] {
        let ptr = IF::phys_to_virt(paddr) as *const PTE;
        unsafe { core::slice::from_raw_parts(ptr, ENTRY_COUNT) }
    }

    fn table_of_mut<'a>(&self, paddr: PhysAddr) -> &'a mut [PTE] {
        let ptr = IF::phys_to_virt(paddr) as *mut PTE;
        unsafe { core::slice::from_raw_parts_mut(ptr, ENTRY_COUNT) }
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

    fn get_entry_mut(&self, vaddr: VirtAddr) -> PagingResult<(&mut PTE, PageSize)> {
        let p3 = if L::LEVELS == 3 {
            self.table_of_mut(self.root_paddr())
        } else if L::LEVELS == 4 {
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

    fn get_entry_mut_or_create(&mut self, page: Page) -> PagingResult<&mut PTE> {
        let vaddr = page.vaddr;
        let p3 = if L::LEVELS == 3 {
            self.table_of_mut(self.root_paddr())
        } else if L::LEVELS == 4 {
            let p4 = self.table_of_mut(self.root_paddr());
            let p4e = &mut p4[p4_index(vaddr)];
            self.next_table_mut_or_create(p4e)?
        } else {
            unreachable!()
        };
        let p3e = &mut p3[p3_index(vaddr)];
        if page.size == PageSize::Size1G {
            return Ok(p3e);
        }

        let p2 = self.next_table_mut_or_create(p3e)?;
        let p2e = &mut p2[p2_index(vaddr)];
        if page.size == PageSize::Size2M {
            return Ok(p2e);
        }

        let p1 = self.next_table_mut_or_create(p2e)?;
        let p1e = &mut p1[p1_index(vaddr)];
        Ok(p1e)
    }

    fn walk_recursive(
        &self,
        table: &[PTE],
        level: usize,
        start_vaddr: usize,
        limit: usize,
        func: &impl Fn(usize, usize, VirtAddr, &PTE),
    ) -> PagingResult {
        let mut n = 0;
        for (i, entry) in table.iter().enumerate() {
            let vaddr = start_vaddr + (i << (12 + (L::LEVELS - 1 - level) * 9));
            if entry.is_present() {
                func(level, i, vaddr, entry);
                if level < L::LEVELS - 1 && !entry.is_huge() {
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

impl<L: PageTableLevels, PTE: GenericPTE, IF: PagingIf> Drop for PageTable64<L, PTE, IF> {
    fn drop(&mut self) {
        for frame in &self.intrm_tables {
            IF::dealloc_frame(*frame);
        }
    }
}

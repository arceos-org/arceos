use alloc::vec::Vec;
use axalloc::PhysPage;
use axerrno::AxResult;
use axhal::{
    mem::{virt_to_phys, VirtAddr, PAGE_SIZE_4K},
    paging::{MappingFlags, PageSize, PageTable},
};
use axio::{Seek, SeekFrom};
use core::ptr::copy_nonoverlapping;
use riscv::asm::sfence_vma;

use crate::MemBackend;

/// A continuous virtual area in user memory.
///
/// NOTE: Cloning a `MapArea` needs allocating new phys pages and modifying a page table. So
/// `Clone` trait won't implemented.
pub struct MapArea {
    pub pages: Vec<Option<PhysPage>>,
    /// 起始虚拟地址
    pub vaddr: VirtAddr,
    pub flags: MappingFlags,
    pub backend: Option<MemBackend>,
}

impl MapArea {
    /// Create a lazy-load area and map it in page table (page fault PTE).
    pub fn new_lazy(
        start: VirtAddr,
        num_pages: usize,
        flags: MappingFlags,
        backend: Option<MemBackend>,
        page_table: &mut PageTable,
    ) -> Self {
        let mut pages = Vec::with_capacity(num_pages);
        for _ in 0..num_pages {
            pages.push(None);
        }

        let _ = page_table
            .map_fault_region(start, num_pages * PAGE_SIZE_4K)
            .unwrap();

        Self {
            pages,
            vaddr: start,
            flags,
            backend,
        }
    }

    /// Allocated an area and map it in page table.
    pub fn new_alloc(
        start: VirtAddr,
        num_pages: usize,
        flags: MappingFlags,
        data: Option<&[u8]>,
        backend: Option<MemBackend>,
        page_table: &mut PageTable,
    ) -> AxResult<Self> {
        let pages = PhysPage::alloc_contiguous(num_pages, PAGE_SIZE_4K, data)?;
        info!(
            "start: {:X?}, size: {:X},  page start: {:X?}",
            start,
            num_pages * PAGE_SIZE_4K,
            pages[0].as_ref().unwrap().start_vaddr
        );
        let _ = page_table
            .map_region(
                start,
                virt_to_phys(pages[0].as_ref().unwrap().start_vaddr),
                num_pages * PAGE_SIZE_4K,
                flags,
                false,
            )
            .unwrap();
        Ok(Self {
            pages,
            vaddr: start,
            flags,
            backend,
        })
    }

    pub fn dealloc(&mut self, page_table: &mut PageTable) {
        page_table.unmap_region(self.vaddr, self.size()).unwrap();
        self.pages.clear();
    }
    /// 如果处理失败，返回false，此时直接退出当前程序
    pub fn handle_page_fault(
        &mut self,
        addr: VirtAddr,
        flags: MappingFlags,
        page_table: &mut PageTable,
    ) -> bool {
        trace!(
            "handling {:?} page fault in area [{:?}, {:?})",
            addr,
            self.vaddr,
            self.end_va()
        );
        assert!(
            self.vaddr <= addr && addr < self.end_va(),
            "Try to handle page fault address out of bound"
        );
        if !self.flags.contains(flags) {
            error!(
                "Try to access {:?} memory addr: {:?} with {:?} flag",
                self.flags, addr, flags
            );
            return false;
        }

        let page_index = (usize::from(addr) - usize::from(self.vaddr)) / PAGE_SIZE_4K;
        if page_index >= self.pages.len() {
            error!("Phys page index out of bound");
            return false;
        }
        if self.pages[page_index].is_some() {
            error!("Page fault in page already loaded");
            return false;
        }

        debug!("page index {}", page_index);

        // Allocate new page
        let mut page = PhysPage::alloc().expect("Error allocating new phys page for page fault");

        debug!(
            "new phys page virtual (offset) address {:?}",
            page.start_vaddr
        );

        // Read data from backend to fill with 0.
        match &mut self.backend {
            Some(backend) => {
                if backend
                    .read_from_seek(
                        SeekFrom::Current((page_index * PAGE_SIZE_4K) as i64),
                        page.as_slice_mut(),
                    )
                    .is_err()
                {
                    warn!("Failed to read from backend to memory");
                    page.fill(0);
                }
            }
            None => page.fill(0),
        };

        // Map newly allocated page in the page_table
        page_table
            .map_overwrite(
                addr.align_down_4k(),
                virt_to_phys(page.start_vaddr),
                axhal::paging::PageSize::Size4K,
                self.flags,
            )
            .expect("Map in page fault handler failed");
        unsafe {
            sfence_vma(0, addr.align_down_4k().into());
        }
        self.pages[page_index] = Some(page);
        true
    }

    /// Sync pages in index back to `self.backend` (if there is one).
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    pub fn sync_page_with_backend(&mut self, page_index: usize) {
        if let Some(page) = &self.pages[page_index] {
            if let Some(backend) = &mut self.backend {
                if backend.writable() {
                    let _ = backend
                        .write_to_seek(
                            SeekFrom::Start((page_index * PAGE_SIZE_4K) as u64),
                            page.as_slice(),
                        )
                        .unwrap();
                }
            }
        } else {
            debug!("Tried to sync an unallocated page");
        }
    }

    /// Deallocate some pages from the start of the area.
    /// This function will unmap them in a page table. You need to flush TLB after this function.
    pub fn shrink_left(&mut self, new_start: VirtAddr, page_table: &mut PageTable) {
        assert!(new_start.is_aligned_4k());

        let delete_size = new_start.as_usize() - self.vaddr.as_usize();
        let delete_pages = delete_size / PAGE_SIZE_4K;

        // move backend offset
        if let Some(backend) = &mut self.backend {
            let _ = backend.seek(SeekFrom::Current(delete_size as i64)).unwrap();
        }

        // remove (dealloc) phys pages
        drop(self.pages.drain(0..delete_pages));

        // unmap deleted pages
        let _ = page_table.unmap_region(self.vaddr, delete_size).unwrap();

        self.vaddr = new_start;
    }

    /// Deallocate some pages from the end of the area.
    /// This function will unmap them in a page table. You need to flush TLB after this function.
    pub fn shrink_right(&mut self, new_end: VirtAddr, page_table: &mut PageTable) {
        assert!(new_end.is_aligned_4k());

        let delete_size = self.end_va().as_usize() - new_end.as_usize();
        let delete_pages = delete_size / PAGE_SIZE_4K;

        // remove (dealloc) phys pages
        drop(
            self.pages
                .drain((self.pages.len() - delete_pages)..self.pages.len()),
        );

        // unmap deleted pages
        let _ = page_table.unmap_region(new_end, delete_size).unwrap();
    }

    /// Split this area into 2.
    pub fn split(&mut self, addr: VirtAddr) -> Self {
        assert!(addr.is_aligned_4k());

        let right_page_count = (self.end_va() - addr.as_usize()).as_usize() / PAGE_SIZE_4K;
        let right_page_range = self.pages.len() - right_page_count..self.pages.len();

        let right_pages = self.pages.drain(right_page_range).collect();

        Self {
            pages: right_pages,
            vaddr: addr,
            flags: self.flags,
            backend: self.backend.as_ref().map(|backend| {
                let mut backend = backend.clone();

                let _ = backend
                    .seek(SeekFrom::Current(
                        (addr.as_usize() - self.vaddr.as_usize()) as i64,
                    ))
                    .unwrap();

                backend
            }),
        }
    }

    /// Split this area into 3.
    pub fn split3(&mut self, start: VirtAddr, end: VirtAddr) -> (Self, Self) {
        assert!(start.is_aligned_4k());
        assert!(end.is_aligned_4k());
        assert!(start < end);
        assert!(self.vaddr < start);
        assert!(end < self.end_va());

        let right_pages = self
            .pages
            .drain(
                self.pages.len() - (self.end_va().as_usize() - end.as_usize()) / PAGE_SIZE_4K
                    ..self.pages.len(),
            )
            .collect();

        let mid_pages = self
            .pages
            .drain(
                self.pages.len() - (self.end_va().as_usize() - start.as_usize()) / PAGE_SIZE_4K
                    ..self.pages.len(),
            )
            .collect();

        let mid = Self {
            pages: mid_pages,
            vaddr: start,
            flags: self.flags,
            backend: self.backend.as_ref().map(|backend| {
                let mut backend = backend.clone();

                let _ = backend
                    .seek(SeekFrom::Current(
                        (start.as_usize() - self.vaddr.as_usize()) as i64,
                    ))
                    .unwrap();

                backend
            }),
        };

        let right = Self {
            pages: right_pages,
            vaddr: end,
            flags: self.flags,
            backend: self.backend.as_ref().map(|backend| {
                let mut backend = backend.clone();

                let _ = backend
                    .seek(SeekFrom::Current(
                        (end.as_usize() - self.vaddr.as_usize()) as i64,
                    ))
                    .unwrap();

                backend
            }),
        };

        (mid, right)
    }

    /// Create a second area in the right part of the area, [self.vaddr, left_end) and
    /// [right_start, self.end_va()).
    /// This function will unmap deleted pages in a page table. You need to flush TLB after calling
    /// this.
    pub fn remove_mid(
        &mut self,
        left_end: VirtAddr,
        right_start: VirtAddr,
        page_table: &mut PageTable,
    ) -> Self {
        assert!(left_end.is_aligned_4k());
        assert!(right_start.is_aligned_4k());
        // We can have left_end == right_start, although it doesn't do anything other than create
        // two areas.
        assert!(left_end <= right_start);

        let delete_size = right_start.as_usize() - left_end.as_usize();
        let delete_range = ((left_end.as_usize() - self.vaddr.as_usize()) / PAGE_SIZE_4K)
            ..((right_start.as_usize() - self.vaddr.as_usize()) / PAGE_SIZE_4K);

        // create a right area
        let pages = self
            .pages
            .drain(((right_start.as_usize() - self.vaddr.as_usize()) / PAGE_SIZE_4K)..)
            .collect();

        let right_area = Self {
            pages,
            vaddr: right_start,
            flags: self.flags,
            backend: self.backend.as_ref().map(|backend| {
                let mut backend = backend.clone();
                let _ = backend
                    .seek(SeekFrom::Current(
                        (right_start.as_usize() - self.vaddr.as_usize()) as i64,
                    ))
                    .unwrap();

                backend
            }),
        };

        // remove pages
        let _ = self.pages.drain(delete_range);

        let _ = page_table.unmap_region(left_end, delete_size).unwrap();

        right_area
    }
}

impl MapArea {
    pub fn size(&self) -> usize {
        self.pages.len() * PAGE_SIZE_4K
    }

    pub fn end_va(&self) -> VirtAddr {
        self.vaddr + self.size()
    }

    pub fn allocated(&self) -> bool {
        self.pages.iter().all(|page| page.is_some())
    }

    pub unsafe fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.vaddr.as_ptr(), self.size()) }
    }

    /// Fill `self` with `byte`.
    pub fn fill(&mut self, byte: u8) {
        self.pages.iter_mut().for_each(|page| {
            if let Some(page) = page {
                page.fill(byte);
            }
        });
    }

    /// If [start, end) overlaps with self.
    pub fn overlap_with(&self, start: VirtAddr, end: VirtAddr) -> bool {
        self.vaddr <= start && start < self.end_va() || start <= self.vaddr && self.vaddr < end
    }

    pub fn contained_in(&self, start: VirtAddr, end: VirtAddr) -> bool {
        start <= self.vaddr && self.end_va() <= end
    }

    pub fn contains(&self, start: VirtAddr, end: VirtAddr) -> bool {
        self.vaddr <= start && end <= self.end_va()
    }

    pub fn strict_contain(&self, start: VirtAddr, end: VirtAddr) -> bool {
        self.vaddr < start && end < self.end_va()
    }

    /// Update area's mapping flags and write it to page table. You need to flush TLB after calling
    /// this function.
    pub fn update_flags(&mut self, flags: MappingFlags, page_table: &mut PageTable) {
        self.flags = flags;
        let _ = page_table
            .update_region(self.vaddr, self.size(), flags)
            .unwrap();
    }

    /// Allocating new phys pages and clone it self.
    /// This function will modify the page table as well.
    pub unsafe fn clone_alloc(&self, page_table: &mut PageTable) -> AxResult<Self> {
        // All the pages have been allocated. Allocate a contiguous area in phys memory.
        if self.allocated() {
            MapArea::new_alloc(
                self.vaddr,
                self.pages.len(),
                self.flags,
                Some(self.as_slice()),
                self.backend.clone(),
                page_table,
            )
        } else {
            let pages: Vec<_> = self
                .pages
                .iter()
                .enumerate()
                .map(|(idx, slot)| {
                    let vaddr = self.vaddr + (idx * PAGE_SIZE_4K);
                    match slot.as_ref() {
                        Some(page) => {
                            let mut new_page = PhysPage::alloc().unwrap();
                            unsafe {
                                copy_nonoverlapping(
                                    page.as_ptr(),
                                    new_page.as_mut_ptr(),
                                    PAGE_SIZE_4K,
                                );
                            }

                            let _ = page_table
                                .map(
                                    vaddr,
                                    virt_to_phys(new_page.start_vaddr),
                                    PageSize::Size4K,
                                    self.flags,
                                )
                                .unwrap();

                            Some(new_page)
                        }
                        None => {
                            let _ = page_table.map_fault(vaddr, PageSize::Size4K).unwrap();
                            None
                        }
                    }
                })
                .collect();
            Ok(Self {
                pages,
                vaddr: self.vaddr,
                flags: self.flags,
                backend: self.backend.clone(),
            })
        }
    }
}

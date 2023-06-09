use allocator::AllocError;
use axerrno::{AxError, AxResult};
use memory_addr::{PhysAddr, VirtAddr};
extern crate alloc;
use crate::{global_allocator, PAGE_SIZE};
use alloc::vec::Vec;

/// A RAII wrapper of contiguous 4K-sized pages.
///
/// It will automatically deallocate the pages when dropped.
#[derive(Debug)]
pub struct GlobalPage {
    start_vaddr: VirtAddr,
    num_pages: usize,
}

impl GlobalPage {
    /// Allocate one 4K-sized page.
    pub fn alloc() -> AxResult<Self> {
        global_allocator()
            .alloc_pages(1, PAGE_SIZE)
            .map(|vaddr| Self {
                start_vaddr: vaddr.into(),
                num_pages: 1,
            })
            .map_err(alloc_err_to_ax_err)
    }

    /// Allocate one 4K-sized page and fill with zero.
    pub fn alloc_zero() -> AxResult<Self> {
        let mut p = Self::alloc()?;
        p.zero();
        Ok(p)
    }

    /// Allocate contiguous 4K-sized pages.
    pub fn alloc_contiguous(num_pages: usize, align_pow2: usize) -> AxResult<Self> {
        global_allocator()
            .alloc_pages(num_pages, align_pow2)
            .map(|vaddr| Self {
                start_vaddr: vaddr.into(),
                num_pages,
            })
            .map_err(alloc_err_to_ax_err)
    }

    /// Get the start virtual address of this page.
    pub fn start_vaddr(&self) -> VirtAddr {
        self.start_vaddr
    }

    /// Get the start physical address of this page.
    pub fn start_paddr<F>(&self, virt_to_phys: F) -> PhysAddr
    where
        F: FnOnce(VirtAddr) -> PhysAddr,
    {
        virt_to_phys(self.start_vaddr)
    }

    /// Get the total size (in bytes) of these page(s).
    pub fn size(&self) -> usize {
        self.num_pages * PAGE_SIZE
    }

    /// Convert to a raw pointer.
    pub fn as_ptr(&self) -> *const u8 {
        self.start_vaddr.as_ptr()
    }

    /// Convert to a mutable raw pointer.
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.start_vaddr.as_mut_ptr()
    }

    /// Fill `self` with `byte`.
    pub fn fill(&mut self, byte: u8) {
        unsafe { core::ptr::write_bytes(self.as_mut_ptr(), byte, self.size()) }
    }

    /// Fill `self` with zero.
    pub fn zero(&mut self) {
        self.fill(0)
    }

    /// Forms a slice that can read data.
    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.as_ptr(), self.size()) }
    }

    /// Forms a mutable slice that can write data.
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), self.size()) }
    }
}

impl Drop for GlobalPage {
    fn drop(&mut self) {
        global_allocator().dealloc_pages(self.start_vaddr.into(), self.num_pages);
    }
}

/// A safe wrapper of a single 4K page.
/// It holds the page's VirtAddr (PhysAddr + offset)
pub struct PhysPage {
    pub start_vaddr: VirtAddr,
}

impl PhysPage {
    pub fn alloc() -> AxResult<Self> {
        global_allocator()
            .alloc_pages(1, PAGE_SIZE)
            .map(|vaddr| Self {
                start_vaddr: vaddr.into(),
            })
            .map_err(alloc_err_to_ax_err)
    }

    pub fn alloc_contiguous(
        num_pages: usize,
        align_pow2: usize,
        data: Option<&[u8]>,
    ) -> AxResult<Vec<Option<Self>>> {
        global_allocator()
            .alloc_pages(num_pages, align_pow2)
            .map(|vaddr| {
                let pages = unsafe {
                    core::slice::from_raw_parts_mut(vaddr as *mut u8, num_pages * PAGE_SIZE)
                };
                pages.fill(0);
                if let Some(data) = data {
                    pages[..data.len()].copy_from_slice(data);
                }

                (0..num_pages)
                    .map(|page_idx| {
                        Some(PhysPage {
                            start_vaddr: (vaddr + page_idx * PAGE_SIZE).into(),
                        })
                    })
                    .collect()
            })
            .map_err(alloc_err_to_ax_err)
    }

    /// Convert to a raw pointer.
    pub fn as_ptr(&self) -> *const u8 {
        self.start_vaddr.as_ptr()
    }

    /// Convert to a mutable raw pointer.
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.start_vaddr.as_mut_ptr()
    }

    /// Forms a slice that can read data.
    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.as_ptr(), PAGE_SIZE) }
    }

    /// Forms a mutable slice that can write data.
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), PAGE_SIZE) }
    }

    /// Fill `self` with `byte`.
    pub fn fill(&mut self, byte: u8) {
        unsafe { core::ptr::write_bytes(self.as_mut_ptr(), byte, PAGE_SIZE) }
    }
}

impl Drop for PhysPage {
    fn drop(&mut self) {
        global_allocator().dealloc_pages(self.start_vaddr.into(), 1);
    }
}

const fn alloc_err_to_ax_err(e: AllocError) -> AxError {
    match e {
        AllocError::InvalidParam | AllocError::MemoryOverlap | AllocError::NotAllocated => {
            AxError::InvalidInput
        }
        AllocError::NoMemory => AxError::NoMemory,
    }
}

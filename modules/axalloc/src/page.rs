#![allow(unused)]
use axerrno::{AxError, AxResult};
use core::{alloc::Layout, ptr::NonNull};
use memory_addr::{PhysAddr, VirtAddr};
use os_memory::err::AllocError;

const PAGE_SIZE: usize = 0x1000;

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
        let layout = Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap();
        os_memory::global_allocator()
            .alloc(layout)
            .map(|vaddr| Self {
                start_vaddr: VirtAddr::from(vaddr.as_ptr() as usize),
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
        let layout = Layout::from_size_align(PAGE_SIZE * num_pages, align_pow2).unwrap();
        os_memory::global_allocator()
            .alloc(layout)
            .map(|vaddr| Self {
                start_vaddr: VirtAddr::from(vaddr.as_ptr() as usize),
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
        let layout = Layout::from_size_align(PAGE_SIZE * self.num_pages, PAGE_SIZE).unwrap();
        unsafe {
            os_memory::global_allocator().dealloc(
                NonNull::new_unchecked(self.start_vaddr().as_mut_ptr()),
                layout,
            );
        }
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

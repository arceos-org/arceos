//! Page table manipulation.

use axalloc::global_allocator;
use lazyinit::LazyInit;
use page_table_multiarch::PagingHandler;

use crate::mem::{MemRegionFlags, PAGE_SIZE_4K, PhysAddr, VirtAddr, phys_to_virt, virt_to_phys};

#[doc(no_inline)]
pub use page_table_multiarch::{MappingFlags, PageSize, PagingError, PagingResult};

impl From<MemRegionFlags> for MappingFlags {
    fn from(f: MemRegionFlags) -> Self {
        let mut ret = Self::empty();
        if f.contains(MemRegionFlags::READ) {
            ret |= Self::READ;
        }
        if f.contains(MemRegionFlags::WRITE) {
            ret |= Self::WRITE;
        }
        if f.contains(MemRegionFlags::EXECUTE) {
            ret |= Self::EXECUTE;
        }
        if f.contains(MemRegionFlags::DEVICE) {
            ret |= Self::DEVICE;
        }
        if f.contains(MemRegionFlags::UNCACHED) {
            ret |= Self::UNCACHED;
        }
        ret
    }
}

/// Implementation of [`PagingHandler`], to provide physical memory manipulation to
/// the [page_table_multiarch] crate.
pub struct PagingHandlerImpl;

impl PagingHandler for PagingHandlerImpl {
    fn alloc_frame() -> Option<PhysAddr> {
        global_allocator()
            .alloc_pages(1, PAGE_SIZE_4K)
            .map(|vaddr| virt_to_phys(vaddr.into()))
            .ok()
    }

    fn dealloc_frame(paddr: PhysAddr) {
        global_allocator().dealloc_pages(phys_to_virt(paddr).as_usize(), 1)
    }

    #[inline]
    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        phys_to_virt(paddr)
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        /// The architecture-specific page table.
        pub type PageTable = page_table_multiarch::x86_64::X64PageTable<PagingHandlerImpl>;
    } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
        /// The architecture-specific page table.
        pub type PageTable = page_table_multiarch::riscv::Sv39PageTable<PagingHandlerImpl>;
    } else if #[cfg(target_arch = "aarch64")]{
        /// The architecture-specific page table.
        pub type PageTable = page_table_multiarch::aarch64::A64PageTable<PagingHandlerImpl>;
    } else if #[cfg(target_arch = "loongarch64")] {
        /// The architecture-specific page table.
        pub type PageTable = page_table_multiarch::loongarch64::LA64PageTable<PagingHandlerImpl>;
    }
}

static KERNEL_PAGE_TABLE_ROOT: LazyInit<PhysAddr> = LazyInit::new();

/// Saves the root physical address of the kernel page table, which may be used
/// on context switch.
pub fn set_kernel_page_table_root(root_paddr: PhysAddr) {
    KERNEL_PAGE_TABLE_ROOT.call_once(|| root_paddr);
    unsafe { crate::arch::write_page_table_root(root_paddr) };
}

/// Get the root physical address of the kernel page table.
///
/// # Panics
///
/// It must be called after [`set_kernel_page_table_root`], otherwise it will panic.
pub fn kernel_page_table_root() -> PhysAddr {
    *KERNEL_PAGE_TABLE_ROOT
        .get()
        .expect("kernel page table not initialized")
}

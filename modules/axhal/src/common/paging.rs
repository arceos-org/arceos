extern crate alloc;

use alloc::alloc::{alloc, dealloc, Layout};

use crate::mem::{phys_to_virt, virt_to_phys, MemRegionFlags};

pub use page_table::{MappingFlags, PagingError, PagingIf, PagingResult};
pub use page_table::{PhysAddr, VirtAddr, PAGE_SIZE_4K};

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
        ret
    }
}

pub struct PagingIfImpl;

impl PagingIf for PagingIfImpl {
    fn alloc_frame() -> Option<PhysAddr> {
        unsafe {
            let layout = Layout::from_size_align_unchecked(PAGE_SIZE_4K, PAGE_SIZE_4K);
            let ptr = alloc(layout);
            if !ptr.is_null() {
                Some(virt_to_phys(ptr as VirtAddr))
            } else {
                None
            }
        }
    }

    fn dealloc_frame(paddr: PhysAddr) {
        unsafe {
            let layout = Layout::from_size_align_unchecked(PAGE_SIZE_4K, PAGE_SIZE_4K);
            let ptr = phys_to_virt(paddr) as *mut u8;
            dealloc(ptr, layout);
        }
    }

    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        phys_to_virt(paddr)
    }
}

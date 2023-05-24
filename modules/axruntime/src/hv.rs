use axalloc::global_allocator;
use axhal::mem::PAGE_SIZE_4K;
use hypercraft::{HostPhysAddr, HyperCraftHal};

/// An empty struct to implementate of `HyperCraftHal`
pub struct HyperCraftHalImpl;

impl HyperCraftHal for HyperCraftHalImpl {
    fn alloc_pages(num_pages: usize) -> Option<hypercraft::HostPhysAddr> {
        global_allocator()
            .alloc_pages(num_pages, PAGE_SIZE_4K)
            .map(|pa| pa as HostPhysAddr)
            .ok()
    }

    fn dealloc_pages(pa: HostPhysAddr, num_pages: usize) {
        global_allocator().dealloc_pages(pa as usize, num_pages);
    }
}

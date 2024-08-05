use core::alloc::Layout;

cfg_alloc! {
    use core::ptr::NonNull;

    pub fn ax_alloc(layout: Layout) -> Option<NonNull<u8>> {
        axalloc::global_allocator().alloc(layout).ok()
    }

    pub fn ax_dealloc(ptr: NonNull<u8>, layout: Layout) {
        axalloc::global_allocator().dealloc(ptr, layout)
    }
}

cfg_dma! {
    pub use axdma::DMAInfo;

    pub unsafe fn ax_alloc_coherent(layout: Layout) -> Option<DMAInfo> {
        axdma::alloc_coherent(layout).ok()
    }

    pub unsafe fn ax_dealloc_coherent(dma: DMAInfo, layout: Layout) {
        axdma::dealloc_coherent(dma, layout)
    }
}

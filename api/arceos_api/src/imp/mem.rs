cfg_alloc! {
    use core::alloc::Layout;
    use core::ptr::NonNull;

    pub fn ax_alloc(layout: Layout) -> Option<NonNull<u8>> {
        if let Ok(vaddr) = axalloc::global_allocator().alloc(layout) {
            Some(vaddr)
        } else {
            None
        }
    }

    pub fn ax_dealloc(ptr: NonNull<u8>, layout: Layout) {
        axalloc::global_allocator().dealloc(ptr, layout)
    }
}

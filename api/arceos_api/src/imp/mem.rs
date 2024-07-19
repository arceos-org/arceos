cfg_alloc! {
    use core::alloc::Layout;
    use core::ptr::NonNull;

    pub fn ax_alloc(layout: Layout) -> Option<NonNull<u8>> {
        unsafe{
            let ptr = alloc::alloc::alloc(layout);
            if ptr.is_null() {
                None
            } else {
                Some(NonNull::new_unchecked(ptr))
            }
        }
    }

    pub fn ax_dealloc(ptr: NonNull<u8>, layout: Layout) {
        unsafe{
            alloc::alloc::dealloc(ptr.as_ptr() as usize as _, layout)
        }
    }
}

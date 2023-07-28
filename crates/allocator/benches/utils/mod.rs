use std::alloc::Layout;
use std::ptr::NonNull;

pub struct MemoryPool {
    ptr: NonNull<u8>,
    layout: Layout,
}

impl MemoryPool {
    pub fn new(size: usize) -> Self {
        let layout = Layout::from_size_align(size, 4096).unwrap();
        let ptr = NonNull::new(unsafe { std::alloc::alloc_zeroed(layout) }).unwrap();
        Self { ptr, layout }
    }

    pub fn as_slice(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.layout.size()) }
    }
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
        unsafe { std::alloc::dealloc(self.ptr.as_ptr(), self.layout) };
    }
}

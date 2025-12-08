use core::alloc::Layout;
use log::{error, info};

#[unsafe(no_mangle)]
pub fn sys_malloc(size: usize, align: usize) -> *mut u8 {
    info!("called sys_malloc with size {} and align {}", size, align);
    if let Ok(layout) = Layout::from_size_align(size, align) {
        unsafe { alloc::alloc::alloc(layout) }
    } else {
        core::ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub fn sys_free(ptr: *mut u8, size: usize, align: usize) {
    info!("called sys_free");
    if let Ok(layout) = Layout::from_size_align(size, align) {
        unsafe { alloc::alloc::dealloc(ptr, layout) }
    } else {
        error!(
            "sys_free called with invalid layout: size {}, align {}",
            size, align
        );
    }
}

#[unsafe(no_mangle)]
pub fn sys_realloc(ptr: *mut u8, size: usize, align: usize, new_size: usize) -> *mut u8 {
    info!("called sys_realloc");
    if let Ok(layout) = Layout::from_size_align(size, align) {
        unsafe { alloc::alloc::realloc(ptr, layout, new_size) }
    } else {
        core::ptr::null_mut()
    }
}

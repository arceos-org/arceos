//! Provides the corresponding malloc(size_t) and free(size_t) when using the C user program.
//!
//! The normal malloc(size_t) and free(size_t) are provided by the library malloc.h, and
//! sys_brk is used internally to apply for memory from the kernel. But in a unikernel like
//! `ArceOS`, we noticed that the heap of the Rust user program is shared with the kernel. In
//! order to maintain consistency, C user programs also choose to share the kernel heap,
//! skipping the sys_brk step.

use alloc::alloc::{alloc, dealloc};
use core::alloc::Layout;
use core::ffi::c_void;

use crate::ctypes;

#[repr(C, align(8))]
struct MemoryControlBlock {
    size: usize,
}

const CTRL_BLK_SIZE: usize = core::mem::size_of::<MemoryControlBlock>();

fn malloc_error() -> *mut c_void {
    crate::errno::set_errno(axerrno::LinuxError::ENOMEM as i32);
    core::ptr::null_mut()
}

/// Allocate memory and return the memory address.
///
/// Returns NULL and sets errno to ENOMEM on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: ctypes::size_t) -> *mut c_void {
    // Allocate `(actual length) + 8`. The lowest 8 Bytes are stored in the actual allocated space size.
    // This is because free(uintptr_t) has only one parameter representing the address,
    // So we need to save in advance to know the size of the memory space that needs to be released
    let Some(total_size) = size.checked_add(CTRL_BLK_SIZE) else {
        return malloc_error();
    };
    let Ok(layout) = Layout::from_size_align(total_size, 8) else {
        return malloc_error();
    };
    unsafe {
        let ptr = alloc(layout).cast::<MemoryControlBlock>();
        if ptr.is_null() {
            return malloc_error();
        }
        ptr.write(MemoryControlBlock { size });
        ptr.add(1).cast()
    }
}

/// Deallocate memory.
///
/// (WARNING) If the address to be released does not match the allocated address, an error should
/// occur, but it will NOT be checked out. This is due to the global allocator `Buddy_system`
/// (currently used) does not check the validity of address to be released.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let ptr = ptr.cast::<MemoryControlBlock>();
    assert!(ptr as usize > CTRL_BLK_SIZE, "free a null pointer");
    unsafe {
        let ptr = ptr.sub(1);
        let size = ptr.read().size;
        let layout = Layout::from_size_align(size + CTRL_BLK_SIZE, 8).unwrap();
        dealloc(ptr.cast(), layout)
    }
}

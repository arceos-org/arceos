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

struct MemoryControlBlock {
    size: usize,
}

const CTRL_BLK_SIZE: usize = core::mem::size_of::<MemoryControlBlock>();

// ISO C requires malloc to return a pointer suitably aligned for any object type
// with fundamental alignment. On most 64-bit architectures (x86_64, aarch64, riscv64,
// loongarch64), max_align_t has an alignment of 16 bytes.
#[cfg(target_pointer_width = "64")]
const MALLOC_ALIGNMENT: usize = 16;

#[cfg(target_pointer_width = "32")]
const MALLOC_ALIGNMENT: usize = 8;

/// Allocate memory and return the memory address.
///
/// Returns NULL on failure per ISO C standard.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: ctypes::size_t) -> *mut c_void {
    // Allocate `(actual length) + CTRL_BLK_SIZE`. The first CTRL_BLK_SIZE bytes store
    // the actual allocated space size. This is because free(uintptr_t) has only one
    // parameter representing the address, so we need to save in advance to know the
    // size of the memory space that needs to be released.
    let layout = match Layout::from_size_align(size + CTRL_BLK_SIZE, MALLOC_ALIGNMENT) {
        Ok(layout) => layout,
        Err(_) => return core::ptr::null_mut(),
    };
    unsafe {
        let ptr = alloc(layout).cast::<MemoryControlBlock>();
        if ptr.is_null() {
            return core::ptr::null_mut();
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
        let layout = Layout::from_size_align(size + CTRL_BLK_SIZE, MALLOC_ALIGNMENT).unwrap();
        dealloc(ptr.cast(), layout)
    }
}

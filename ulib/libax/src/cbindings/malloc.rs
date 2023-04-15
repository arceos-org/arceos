//! Provides the corresponding malloc(size_t) and free(size_t) when using the C user program.
//!
//! The normal malloc(size_t) and free(size_t) are provided by the library malloc.h, and
//! sys_brk is used internally to apply for memory from the kernel. But in a unikernel like
//! `ArceOS`, we noticed that the heap of the Rust user program is shared with the kernel. In
//! order to maintain consistency, C user programs also choose to share the kernel heap,
//! skipping the sys_brk step.

use core::result::Result::{Err, Ok};
use core::{ffi::c_void, mem::size_of};

const BYTES_OF_USIZE: usize = 0x8;

struct MemoryControlBlock {
    size: usize,
}

/// Allocate memory and return the memory address.
///
/// Returns 0 on failure (the current implementation does not trigger an exception)
#[no_mangle]
pub unsafe extern "C" fn ax_malloc(size: usize) -> *mut c_void {
    // Allocate `(actual length) + 8`. The lowest 8 Bytes are stored in the actual allocated space size.
    // This is because free(uintptr_t) has only one parameter representing the address,
    // So we need to save in advance to know the size of the memory space that needs to be released
    match axalloc::global_allocator().alloc(size + size_of::<MemoryControlBlock>(), BYTES_OF_USIZE)
    {
        Ok(addr) => {
            let control_block = unsafe { &mut *(addr as *mut MemoryControlBlock) };
            control_block.size = size;
            (addr + 8) as *mut c_void
        }
        Err(_) => core::ptr::null_mut(),
    }
}

/// Deallocate memory.
///
/// (WARNING) If the address to be released does not match the allocated address, an error should
/// occur, but it will NOT be checked out. This is due to the global allocator `Buddy_system`
/// (currently used) does not check the validity of address to be released.
#[no_mangle]
pub unsafe extern "C" fn ax_free(addr: *mut c_void) {
    let size = {
        let control_block = unsafe {
            &mut *((addr as usize - size_of::<MemoryControlBlock>()) as *mut MemoryControlBlock)
        };
        control_block.size
    };
    axalloc::global_allocator().dealloc(
        addr as usize - size_of::<MemoryControlBlock>(),
        size + size_of::<MemoryControlBlock>(),
        BYTES_OF_USIZE,
    )
}

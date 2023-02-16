//! 提供使用 C 用户程序时对应的 malloc(size_t) 和 free(size_t)
//!
//! 正常的 malloc(size_t) 和 free(size_t) 由库 malloc.h 提供，
//! 内部使用 sys_brk 来实现向内核申请内存。但在 ArceOS 这样一个
//! unikernel 中，我们注意到 Rust 用户程序的堆是和内核共用的。
//! 为了保持一致性，C 的用户程序也选择共用内核堆，跳过 sys_brk
//! 这一步。
#![no_std]
#![feature(c_size_t)]

extern crate alloc;
extern crate axalloc;

use core::{ffi::{c_size_t, c_void}, mem::size_of};
use core::result::Result::{Ok, Err};

const BYTES_OF_USIZE: usize = 0x8;

struct MemoryControlBlock {
    size: usize,
}

/// 申请分配一段内存，返回内存地址。
///
/// 如失败，则返回 0（目前的实现不会触发 exception）
#[no_mangle]
pub extern "C" fn malloc(size: c_size_t) -> *mut c_void {
    // 分配实际长度 + 8，最低 8 Byte 存入实际分配的空间大小。
    // 这样做是因为，free(uintptr_t) 只有一个参数表示地址，
    // 所以需要事先保存才知道需要释放的内存空间大小
    match axalloc::global_allocator().alloc(size + size_of::<MemoryControlBlock>(), BYTES_OF_USIZE)
    {
        Ok(addr) => {
            let control_block = unsafe { &mut *(addr as *mut MemoryControlBlock) };
            control_block.size = size;
            (addr + 8) as *mut c_void
        }
        Err(_) => 0 as *mut c_void,
    }
}

/// 释放一段内存。
///
/// (WARNING)如释放的地址和分配的不符，则释放时会出错，而且不会被检查出来。
/// 这是由于内存分配目前使用的 Buddy_system 没有足够的释放时检查
#[no_mangle]
pub extern "C" fn free(addr: *mut c_void) {
    let size = {
        let control_block =
            unsafe { &mut *((addr as usize - size_of::<MemoryControlBlock>()) as *mut MemoryControlBlock) };
        control_block.size
    };
    axalloc::global_allocator().dealloc(
        addr as usize - size_of::<MemoryControlBlock>(),
        size + size_of::<MemoryControlBlock>(),
        BYTES_OF_USIZE,
    )
}

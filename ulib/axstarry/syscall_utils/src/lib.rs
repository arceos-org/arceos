//! 存储系统调用的相关类型
#![cfg_attr(all(not(test), not(doc)), no_std)]
mod ctypes;
use axerrno::LinuxError;
use axlog::error;
pub use ctypes::*;
pub type SyscallResult = Result<isize, LinuxError>;

pub fn deal_result(result: SyscallResult) -> isize {
    match result {
        Ok(x) => x,
        Err(error) => error.code() as isize,
    }
}

pub type SyscallError = axerrno::LinuxError;
#[allow(unused)]
pub(crate) unsafe fn get_str_len(start: *const u8) -> usize {
    let mut ptr = start as usize;
    while *(ptr as *const u8) != 0 {
        ptr += 1;
    }
    ptr - start as usize
}

#[allow(unused)]
pub(crate) unsafe fn raw_ptr_to_ref_str(start: *const u8) -> &'static str {
    let len = get_str_len(start);
    // 因为这里直接用用户空间提供的虚拟地址来访问，所以一定能连续访问到字符串，不需要考虑物理地址是否连续
    let slice = core::slice::from_raw_parts(start, len);
    if let Ok(s) = core::str::from_utf8(slice) {
        s
    } else {
        error!("not utf8 slice");
        for c in slice {
            error!("{c} ");
        }
        error!("");
        &"p"
    }
}

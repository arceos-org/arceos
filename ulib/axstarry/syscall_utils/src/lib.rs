//! 存储系统调用的相关类型
#![cfg_attr(all(not(test), not(doc)), no_std)]
mod ctypes;
use axerrno::LinuxError;
pub use ctypes::*;
pub type SyscallResult = Result<isize, LinuxError>;
mod file;
pub use file::*;
pub fn deal_result(result: SyscallResult) -> isize {
    match result {
        Ok(x) => x,
        Err(error) => -(error.code() as isize),
    }
}

pub type SyscallError = axerrno::LinuxError;

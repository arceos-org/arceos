//! The entry of syscall, which will distribute the syscall to the corresponding function
#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]
#![feature(naked_functions)]
#![feature(stmt_expr_attributes)]
extern crate alloc;
/// 需要手动引入这个库，否则会报错：`#[panic_handler]` function required, but not found.
extern crate axruntime;

mod trap;

mod syscall_fs;

mod syscall_mem;

#[cfg(feature = "net")]
mod syscall_net;

mod syscall_task;

mod syscall;

mod ctypes;
use ctypes::*;

pub use axprocess::{
    link::{create_link, FilePath},
    wait_pid, Process,
};
pub use axprocess::{yield_now_task, PID2PC};

mod api;
pub use api::*;

#[cfg(feature = "ext4fs")]
#[allow(unused_imports)]
use axlibc::ax_open;

/// Accept the result of a syscall, and return the isize to the user
pub(crate) fn deal_result(result: SyscallResult) -> isize {
    match result {
        Ok(x) => x,
        Err(error) => -(error.code() as isize),
    }
}

/// The error of a syscall, which is a `LinuxError`
pub type SyscallError = axerrno::LinuxError;

/// The result of a syscall
///
/// * `Ok(x)` - The syscall is successful, and the return value is `x`
///
/// * `Err(error)` - The syscall failed, and the error is related to `linux_error`
pub type SyscallResult = Result<isize, SyscallError>;

//! This crate includes syscalls that need `multitask` feature

use crate::debug;
use crate::task::exit;
use axerrno::{LinuxError, LinuxResult};
use core::ffi::c_int;

#[cfg(feature = "multitask")]
use crate::task::current;

/// Exit current task
#[no_mangle]
pub unsafe extern "C" fn ax_exit(exit_code: c_int) -> ! {
    exit(exit_code)
}

/// Get current thread ID
#[cfg(feature = "multitask")]
#[no_mangle]
pub unsafe extern "C" fn ax_getpid() -> c_int {
    ax_call_body!(ax_getpid, {
        let pid = current().id().as_u64() as c_int;
        debug!("getpid return {}", pid);
        return Ok(pid);
    })
}

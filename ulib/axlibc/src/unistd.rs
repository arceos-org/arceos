use arceos_posix_api::{sys_exit, sys_getpid};
use core::ffi::c_int;

/// Get current thread ID.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getpid() -> c_int {
    sys_getpid()
}

/// Abort the current process.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn abort() -> ! {
    panic!()
}

/// Exits the current thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn exit(exit_code: c_int) -> ! {
    sys_exit(exit_code)
}

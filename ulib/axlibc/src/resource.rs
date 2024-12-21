use core::ffi::c_int;

use arceos_posix_api::{sys_getrlimit, sys_setrlimit};

use crate::utils::e;

/// Get resource limitations
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getrlimit(resource: c_int, rlimits: *mut crate::ctypes::rlimit) -> c_int {
    e(sys_getrlimit(resource, rlimits))
}

/// Set resource limitations
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setrlimit(resource: c_int, rlimits: *mut crate::ctypes::rlimit) -> c_int {
    e(sys_setrlimit(resource, rlimits))
}

use arceos_posix_api::{sys_clock_gettime, sys_nanosleep};
use core::ffi::c_int;

use crate::{ctypes, utils::e};

/// Get clock time since booting
#[unsafe(no_mangle)]
pub unsafe extern "C" fn clock_gettime(clk: ctypes::clockid_t, ts: *mut ctypes::timespec) -> c_int {
    e(sys_clock_gettime(clk, ts))
}

/// Sleep some nanoseconds
///
/// TODO: should be woken by signals, and set errno
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nanosleep(
    req: *const ctypes::timespec,
    rem: *mut ctypes::timespec,
) -> c_int {
    e(sys_nanosleep(req, rem))
}

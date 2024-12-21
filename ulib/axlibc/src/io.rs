use core::ffi::{c_int, c_void};

use arceos_posix_api::{sys_read, sys_write, sys_writev};

use crate::{ctypes, utils::e};

/// Read data from the file indicated by `fd`.
///
/// Return the read size if success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn read(fd: c_int, buf: *mut c_void, count: usize) -> ctypes::ssize_t {
    e(sys_read(fd, buf, count) as _) as _
}

/// Write data to the file indicated by `fd`.
///
/// Return the written size if success.
#[unsafe(no_mangle)]
#[cfg(not(test))]
pub unsafe extern "C" fn write(fd: c_int, buf: *const c_void, count: usize) -> ctypes::ssize_t {
    e(sys_write(fd, buf, count) as _) as _
}

/// Write a vector.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn writev(
    fd: c_int,
    iov: *const ctypes::iovec,
    iocnt: c_int,
) -> ctypes::ssize_t {
    e(sys_writev(fd, iov, iocnt) as _) as _
}

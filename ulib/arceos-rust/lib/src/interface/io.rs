use core::{
    ffi::c_char,
    sync::atomic::{AtomicI32, Ordering},
};

use arceos_posix_api::ctypes::iovec;
use log::info;

/// Global errno variable for the hermit ABI.
///
/// Hermit std's `io::Error::last_os_error()` calls `sys_get_errno()` to
/// retrieve the last error. POSIX-style calls like `poll()` return -1 on
/// error and store the actual error code here.
pub(crate) static ERRNO: AtomicI32 = AtomicI32::new(0);

/// Set the global errno value (used by POSIX-style wrappers like sys_poll).
#[inline]
pub(crate) fn set_errno(e: i32) {
    ERRNO.store(e, Ordering::Relaxed);
}

/// Get the last error number from the thread local storage.
///
/// Called by hermit std's `io::Error::last_os_error()`.
#[unsafe(no_mangle)]
pub fn sys_get_errno() -> i32 {
    let e = ERRNO.load(Ordering::Relaxed);
    info!("called sys_get_errno => {}", e);
    e
}

#[unsafe(no_mangle)]
pub fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
    info!("called sys_write");
    arceos_posix_api::sys_write(fd, buf as _, count)
}

#[unsafe(no_mangle)]
pub fn sys_writev(fd: i32, iov: *const iovec, iovcnt: usize) -> isize {
    info!("called sys_writev");
    unsafe { arceos_posix_api::sys_writev(fd, iov, iovcnt as _) }
}

#[cfg(feature = "fd")]
#[unsafe(no_mangle)]
pub fn sys_close(fd: i32) -> i32 {
    info!("called sys_close");
    arceos_posix_api::sys_close(fd) as _
}

#[unsafe(no_mangle)]
pub fn sys_read(fd: i32, buf: *mut u8, len: usize) -> isize {
    info!("called sys_read");
    arceos_posix_api::sys_read(fd, buf as _, len)
}

#[cfg(feature = "fs")]
#[unsafe(no_mangle)]
pub fn sys_lseek(fd: i32, offset: isize, whence: i32) -> isize {
    info!("called sys_lseek");
    arceos_posix_api::sys_lseek(fd, offset as _, whence) as _
}

#[cfg(feature = "fs")]
#[unsafe(no_mangle)]
pub fn sys_open(name: *const c_char, flags: i32, mode: i32) -> i32 {
    info!("called sys_open");
    arceos_posix_api::sys_open(name, flags, mode as _)
}

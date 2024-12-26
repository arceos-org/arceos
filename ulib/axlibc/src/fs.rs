use core::ffi::{c_char, c_int};

use arceos_posix_api::{
    sys_fstat, sys_getcwd, sys_lseek, sys_lstat, sys_open, sys_rename, sys_stat,
};

use crate::{ctypes, utils::e};

/// Open a file by `filename` and insert it into the file descriptor table.
///
/// Return its index in the file table (`fd`). Return `EMFILE` if it already
/// has the maximum number of files open.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ax_open(
    filename: *const c_char,
    flags: c_int,
    mode: ctypes::mode_t,
) -> c_int {
    e(sys_open(filename, flags, mode))
}

/// Set the position of the file indicated by `fd`.
///
/// Return its position after seek.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lseek(fd: c_int, offset: ctypes::off_t, whence: c_int) -> ctypes::off_t {
    e(sys_lseek(fd, offset, whence) as _) as _
}

/// Get the file metadata by `path` and write into `buf`.
///
/// Return 0 if success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stat(path: *const c_char, buf: *mut ctypes::stat) -> c_int {
    e(sys_stat(path, buf))
}

/// Get file metadata by `fd` and write into `buf`.
///
/// Return 0 if success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fstat(fd: c_int, buf: *mut ctypes::stat) -> c_int {
    e(sys_fstat(fd, buf))
}

/// Get the metadata of the symbolic link and write into `buf`.
///
/// Return 0 if success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lstat(path: *const c_char, buf: *mut ctypes::stat) -> c_int {
    e(sys_lstat(path, buf) as _)
}

/// Get the path of the current directory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getcwd(buf: *mut c_char, size: usize) -> *mut c_char {
    sys_getcwd(buf, size)
}

/// Rename `old` to `new`
/// If new exists, it is first removed.
///
/// Return 0 if the operation succeeds, otherwise return -1.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rename(old: *const c_char, new: *const c_char) -> c_int {
    e(sys_rename(old, new))
}

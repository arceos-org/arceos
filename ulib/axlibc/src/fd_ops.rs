use crate::{ctypes, utils::e};
use arceos_posix_api::{sys_close, sys_dup, sys_dup2, sys_fcntl};
use axerrno::LinuxError;
use core::ffi::c_int;

/// Close a file by `fd`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn close(fd: c_int) -> c_int {
    e(sys_close(fd))
}

/// Duplicate a file descriptor.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dup(old_fd: c_int) -> c_int {
    e(sys_dup(old_fd))
}

/// Duplicate a file descriptor, use file descriptor specified in `new_fd`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dup2(old_fd: c_int, new_fd: c_int) -> c_int {
    e(sys_dup2(old_fd, new_fd))
}

/// Duplicate a file descriptor, the caller can force the close-on-exec flag to
/// be set for the new file descriptor by specifying `O_CLOEXEC` in flags.
///
/// If oldfd equals newfd, then `dup3()` fails with the error `EINVAL`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dup3(old_fd: c_int, new_fd: c_int, flags: c_int) -> c_int {
    if old_fd == new_fd {
        return e((LinuxError::EINVAL as c_int).wrapping_neg());
    }
    let r = e(sys_dup2(old_fd, new_fd));
    if r < 0 {
        r
    } else {
        if flags as u32 & ctypes::O_CLOEXEC != 0 {
            e(sys_fcntl(
                new_fd,
                ctypes::F_SETFD as c_int,
                ctypes::FD_CLOEXEC as usize,
            ));
        }
        new_fd
    }
}

/// Manipulate file descriptor.
///
/// TODO: `SET/GET` command is ignored
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ax_fcntl(fd: c_int, cmd: c_int, arg: usize) -> c_int {
    e(sys_fcntl(fd, cmd, arg))
}

use core::ffi::c_int;

use super::{ax_write, ctypes};
use axerrno::LinuxError;

/// `writev` implementation
#[no_mangle]
pub unsafe extern "C" fn ax_writev(
    fd: c_int,
    iov: *const ctypes::iovec,
    iocnt: c_int,
) -> ctypes::ssize_t {
    debug!("ax_writev <= fd: {}", fd);
    ax_call_body!(ax_writev, {
        if !(0..=1024).contains(&iocnt) {
            return Err(LinuxError::EINVAL);
        }

        let iovs = core::slice::from_raw_parts(iov, iocnt as usize);
        let mut ret = 0;
        for iov in iovs.iter() {
            ret += ax_write(fd, iov.iov_base, iov.iov_len);
        }

        Ok(ret)
    })
}

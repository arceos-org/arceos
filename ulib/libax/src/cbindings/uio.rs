use alloc::vec::Vec;
use core::ffi::{c_int, c_void};

use super::{ax_write, ctypes, errno::set_errno};
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
            set_errno(LinuxError::EINVAL as _);
            return Err(LinuxError::EINVAL);
        }
        let iovs = core::slice::from_raw_parts(iov, iocnt as usize);
        let mut vec = Vec::new();
        for iov in iovs.iter() {
            vec.extend_from_slice(core::slice::from_raw_parts_mut(
                iov.iov_base as *mut u8,
                iov.iov_len,
            ));
        }

        Ok(ax_write(fd, vec.as_ptr() as *const c_void, vec.len()))
    })
}

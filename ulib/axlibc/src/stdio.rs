use core::ffi::{c_char, c_int};

use axerrno::LinuxError;
use axstd::io::{self, Write};

#[cfg(feature = "fd")]
use {alloc::sync::Arc, axerrno::LinuxResult, axio::PollState, axstd::io::Read};

/// Print a string to the global standard output stream.
#[no_mangle]
pub unsafe extern "C" fn ax_print_str(buf: *const c_char, count: usize) -> c_int {
    ax_call_body_no_debug!({
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }

        let bytes = unsafe { core::slice::from_raw_parts(buf as *const u8, count as _) };
        let len = io::stdout().write(bytes)?;
        Ok(len as c_int)
    })
}

/// Print a string to the global standard output stream. Add a line break.
#[no_mangle]
pub unsafe extern "C" fn ax_println_str(buf: *const c_char, count: usize) -> c_int {
    ax_call_body_no_debug!({
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }

        let bytes = unsafe { core::slice::from_raw_parts(buf as *const u8, count as _) };
        let mut stdout = io::stdout().lock();
        let len = stdout.write(bytes)?;
        let len = stdout.write(b"\n")? + len;
        Ok(len as c_int)
    })
}

#[cfg(feature = "fd")]
impl super::fd_ops::FileLike for axstd::io::Stdin {
    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        Ok(self.lock().read(buf)?)
    }

    fn write(&self, _buf: &[u8]) -> LinuxResult<usize> {
        Err(LinuxError::EPERM)
    }

    fn stat(&self) -> LinuxResult<super::ctypes::stat> {
        let st_mode = 0o20000 | 0o440u32; // S_IFCHR | r--r-----
        Ok(super::ctypes::stat {
            st_ino: 1,
            st_nlink: 1,
            st_mode,
            ..Default::default()
        })
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
        })
    }

    fn set_nonblocking(&self, _nonblocking: bool) -> LinuxResult {
        Ok(())
    }
}

#[cfg(feature = "fd")]
impl super::fd_ops::FileLike for axstd::io::Stdout {
    fn read(&self, _buf: &mut [u8]) -> LinuxResult<usize> {
        Err(LinuxError::EPERM)
    }

    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        Ok(self.lock().write(buf)?)
    }

    fn stat(&self) -> LinuxResult<super::ctypes::stat> {
        let st_mode = 0o20000 | 0o220u32; // S_IFCHR | -w--w----
        Ok(super::ctypes::stat {
            st_ino: 1,
            st_nlink: 1,
            st_mode,
            ..Default::default()
        })
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
        })
    }

    fn set_nonblocking(&self, _nonblocking: bool) -> LinuxResult {
        Ok(())
    }
}

use axerrno::LinuxError;
use core::ffi::{c_char, c_int};

/// The global errno variable.
#[cfg_attr(feature = "tls", thread_local)]
#[unsafe(no_mangle)]
#[allow(non_upper_case_globals)]
pub static mut errno: c_int = 0;

pub fn set_errno(code: i32) {
    unsafe {
        errno = code;
    }
}

/// Returns a pointer to the global errno variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __errno_location() -> *mut c_int {
    core::ptr::addr_of_mut!(errno)
}

/// Returns a pointer to the string representation of the given error code.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strerror(e: c_int) -> *mut c_char {
    #[allow(non_upper_case_globals)]
    static mut strerror_buf: [u8; 256] = [0; 256]; // TODO: thread safe

    let err_str = if e == 0 {
        "Success"
    } else {
        LinuxError::try_from(e)
            .map(|e| e.as_str())
            .unwrap_or("Unknown error")
    };
    unsafe {
        strerror_buf[..err_str.len()].copy_from_slice(err_str.as_bytes());
        &raw mut strerror_buf as *mut c_char
    }
}

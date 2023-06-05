#![allow(dead_code)]
#![allow(unused_macros)]

use axerrno::{LinuxError, LinuxResult};
use core::ffi::{c_char, CStr};

pub fn char_ptr_to_str<'a>(str: *const c_char) -> LinuxResult<&'a str> {
    if str.is_null() {
        Err(LinuxError::EFAULT)
    } else {
        unsafe { CStr::from_ptr(str) }
            .to_str()
            .map_err(|_| LinuxError::EINVAL)
    }
}

macro_rules! ax_call_body {
    ($fn: ident, $($stmt: tt)*) => {{
        #[allow(clippy::redundant_closure_call)]
        let res = (|| -> LinuxResult<_> { $($stmt)* })();
        if res.is_err() {
            $crate::info!(concat!(stringify!($fn), " => {:?}"),  res);
        } else {
            $crate::debug!(concat!(stringify!($fn), " => {:?}"), res);
        }
        match res {
            Ok(v) => v as _,
            Err(e) => {
                super::errno::set_errno(e.code());
                -1 as _
            }
        }
    }};
}

macro_rules! ax_call_body_no_debug {
    ($($stmt: tt)*) => {{
        #[allow(clippy::redundant_closure_call)]
        let res = (|| -> LinuxResult<_> { $($stmt)* })();
        match res {
            Ok(v) => v as _,
            Err(e) => {
                super::errno::set_errno(e.code());
                -1 as _
            }
        }
    }};
}

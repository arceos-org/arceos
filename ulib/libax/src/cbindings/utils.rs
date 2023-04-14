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
        let res = (|| -> LinuxResult<_> { $($stmt)* })();
        if res.is_err() {
            $crate::info!(concat!(stringify!($fn), " => {:?}"),  res);
        } else {
            $crate::debug!(concat!(stringify!($fn), " => {:?}"), res);
        }
        match res {
            Ok(v) => v as _,
            Err(e) => -e.code() as _,
        }
    }};
}

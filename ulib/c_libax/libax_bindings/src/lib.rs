#![no_std]
#![allow(clippy::missing_safety_doc)]

use core::ffi::{c_char, c_int};

#[macro_use]
extern crate libax;

#[no_mangle]
pub unsafe extern "C" fn ax_print_str(buf: *const c_char, count: usize) -> c_int {
    let str = core::slice::from_raw_parts(buf as *const u8, count as _);
    print!("{}", core::str::from_utf8_unchecked(str));
    str.len() as _
}

#[no_mangle]
pub extern "C" fn ax_panic() -> ! {
    panic!()
}

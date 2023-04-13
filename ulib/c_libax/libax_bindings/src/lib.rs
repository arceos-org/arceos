#![no_std]
#![allow(clippy::missing_safety_doc)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[macro_use]
#[allow(unused_imports)]
extern crate log;

#[macro_use]
mod utils;

#[cfg(feature = "fs")]
mod fs;
#[cfg(feature = "alloc")]
mod malloc;

/// cbindgen:ignore
#[path = "../ctypes_gen.rs"]
#[allow(dead_code, non_camel_case_types)]
mod ctypes;

use core::ffi::{c_char, c_int};
use libax::io::Write;

#[no_mangle]
pub extern "C" fn ax_srand(seed: u32) {
    libax::rand::srand(seed);
}

#[no_mangle]
pub extern "C" fn ax_rand_u32() -> u32 {
    libax::rand::rand_u32()
}

#[no_mangle]
pub unsafe extern "C" fn ax_print_str(buf: *const c_char, count: usize) -> c_int {
    if buf.is_null() {
        return -axerrno::LinuxError::EFAULT.code();
    }
    let bytes = core::slice::from_raw_parts(buf as *const u8, count as _);
    libax::io::stdout().write(bytes).unwrap() as _
}

#[no_mangle]
#[panic_handler]
pub extern "C" fn ax_panic() -> ! {
    panic!()
}

#[cfg(feature = "alloc")]
pub use self::malloc::{ax_free, ax_malloc};

#[cfg(feature = "fs")]
pub use self::fs::{
    ax_close, ax_fstat, ax_getcwd, ax_lseek, ax_lstat, ax_open, ax_read, ax_stat, ax_write,
};

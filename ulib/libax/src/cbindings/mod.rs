//! Exported C bindings, to call ArceOS funtions from C code.

#![allow(clippy::missing_safety_doc)]

#[macro_use]
mod utils;

#[cfg(feature = "fs")]
mod fs;
#[cfg(feature = "alloc")]
mod malloc;

/// cbindgen:ignore
#[rustfmt::skip]
#[path = "./ctypes_gen.rs"]
#[allow(dead_code, non_camel_case_types)]
mod ctypes;

use crate::io::Write;
use core::ffi::{c_char, c_int};

/// Sets the seed for the random number generator.
#[no_mangle]
pub unsafe extern "C" fn ax_srand(seed: u32) {
    crate::rand::srand(seed);
}

/// Returns a 32-bit unsigned pseudo random interger.
#[no_mangle]
pub unsafe extern "C" fn ax_rand_u32() -> u32 {
    crate::rand::rand_u32()
}

/// Print a string to the global standard output stream.
#[no_mangle]
pub unsafe extern "C" fn ax_print_str(buf: *const c_char, count: usize) -> c_int {
    if buf.is_null() {
        return -axerrno::LinuxError::EFAULT.code();
    }
    let bytes = unsafe { core::slice::from_raw_parts(buf as *const u8, count as _) };
    crate::io::stdout().write(bytes).unwrap() as _
}

/// Abort the current process.
#[no_mangle]
pub unsafe extern "C" fn ax_panic() -> ! {
    panic!()
}

#[cfg(feature = "alloc")]
pub use self::malloc::{ax_free, ax_malloc};

#[cfg(feature = "fs")]
pub use self::fs::{
    ax_close, ax_fstat, ax_getcwd, ax_lseek, ax_lstat, ax_open, ax_read, ax_stat, ax_write,
};

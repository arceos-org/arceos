#![no_std]
#![allow(clippy::missing_safety_doc)]

#[cfg(feature = "alloc")]
mod malloc;

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
    let bytes = core::slice::from_raw_parts(buf as *const u8, count as _);
    libax::io::stdout().write(bytes).unwrap() as _
}

#[no_mangle]
pub extern "C" fn ax_panic() -> ! {
    panic!()
}

#[cfg(feature = "alloc")]
pub use malloc::{ax_free, ax_malloc};

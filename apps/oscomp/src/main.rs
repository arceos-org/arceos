#![no_std]
#![no_main]
// extern crate axstarry;

use syscall_entry::run_testcases;

#[no_mangle]
fn main() {
    let _ = axlibc::ax_open as usize;
    run_testcases("sdcard");
}

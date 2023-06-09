#![no_std]
#![no_main]

// Build with libax to get global_allocator and stuff.

// extern crate alloc;
extern crate libax;
// use alloc::string::ToString;
use libax::test::run_testcases;
#[no_mangle]
fn main() -> i32 {
    run_testcases("libc-static");
    0
}

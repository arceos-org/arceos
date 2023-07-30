#![no_std]
#![no_main]

// Build with libax to get global_allocator and stuff.

extern crate libax;

use libax::test::run_iperf;

#[no_mangle]
fn main() -> i32 {
    run_iperf();
    0
}

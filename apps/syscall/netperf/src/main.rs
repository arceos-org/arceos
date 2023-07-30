#![no_std]
#![no_main]

// Build with libax to get global_allocator and stuff.

extern crate libax;

use libax::test::run_netperf;

#[no_mangle]
fn main() -> i32 {
    run_netperf();
    0
}

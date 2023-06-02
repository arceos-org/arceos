#![no_std]
#![no_main]

// Build with libax to get global_allocator and stuff.

// extern crate alloc;
extern crate libax;
// use alloc::string::ToString;
use libax::test::run_testcases;
#[no_mangle]
fn main() -> i32 {
    // let name = "execve".to_string();
    // let args = [name].to_vec();
    // run_testcase(args).unwrap();
    run_testcases();
    0
}

#![no_std]
#![no_main]
// extern crate axstarry;

use axstarry::run_testcases;

#[no_mangle]
fn main() {
    run_testcases("ostrain");
}

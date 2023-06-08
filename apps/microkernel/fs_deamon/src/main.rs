#![no_std]
#![no_main]

mod fs;

#[macro_use]
extern crate libax;

#[no_mangle]
fn main() {
    fs::init_fs();
    fs::run();
}

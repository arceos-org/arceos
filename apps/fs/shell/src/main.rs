#![no_std]
#![no_main]

#[macro_use]
extern crate libax;

#[no_mangle]
fn main() {
    print!("arceos:/$ ");
    loop {}
}

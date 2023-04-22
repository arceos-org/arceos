#![no_std]
#![no_main]

#[macro_use]
extern crate libax;


mod test_sleep;

#[no_mangle]
fn main() {
    libax::println!("Hello, testcases!");
    test_sleep::main();
}

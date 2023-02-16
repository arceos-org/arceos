#![no_std]
#![no_main]

#[macro_use]
extern crate axruntime;
extern crate axalloc_c;

extern "C" {
    fn c_main() -> i32;
}

#[no_mangle]
fn main() {
    println!("Hello, C world!");
    let return_code = unsafe { c_main() };
    println!("return code = {}", return_code);
}

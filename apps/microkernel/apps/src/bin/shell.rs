#![no_std]
#![no_main]

use microkernel_apps::shell;

extern crate libax;

#[no_mangle]
fn main() {
    shell::main();
}

#![no_std]
#![no_main]

use microkernel_apps::http_client;

extern crate libax;

#[no_mangle]
fn main() {
    http_client::main();
}

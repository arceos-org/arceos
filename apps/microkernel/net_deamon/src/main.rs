#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
mod tcp;
mod udp;

#[no_mangle]
fn main() {
    axnet::user_init();
    tcp::start_tcp();
}

#![no_std]

mod tcp;
mod udp;
pub use tcp::start_tcp;

#[macro_use]
extern crate libax;

pub fn init() {
    axnet::user_init();
}

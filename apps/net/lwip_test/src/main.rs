#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
use libax::net::TcpSocket;

#[no_mangle]
fn main() {
    println!("Hello, lwip test!");
}

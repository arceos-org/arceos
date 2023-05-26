#![no_std]
#![no_main]

use libax::task::{spawn, yield_now};
use net_deamon::start_tcp;

#[macro_use]
extern crate libax;

mod http_client;
mod http_server;

#[no_mangle]
fn main() {
    println!("Hello world!");

    println!("Start TCP deamon");

    net_deamon::init();

    spawn(|| start_tcp());

    spawn(|| http_server::main());

    spawn(|| http_client::main());

    loop {
        yield_now();
    }
}

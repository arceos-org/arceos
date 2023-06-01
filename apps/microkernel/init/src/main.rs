#![no_std]
#![no_main]

use libax::{task::{spawn, yield_now}, process::fork};
use net_deamon::start_tcp;

#[macro_use]
extern crate libax;

mod http_client;
mod http_server;

fn fake_exec(f: fn()) {
    match fork() {
        pid if pid > 0 => {
            return;            
        }
        0 => {
            f();
            loop { yield_now() }
        }
        _ => {
            panic!("Error fork()");
        }
    }

}
fn run_net_deamon() {
    net_deamon::init();
    start_tcp();
}

#[no_mangle]
fn main() {
    println!("Hello world!");

    println!("Start TCP deamon");

 

    fake_exec(run_net_deamon);
    fake_exec(http_client::main);
    fake_exec(http_server::main);
    
    loop {
        yield_now();
    }
}

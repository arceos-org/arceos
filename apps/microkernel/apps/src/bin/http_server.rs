#![no_std]
#![no_main]

use libax::{process::fork, task::exit};
use microkernel_apps::http_server;

extern crate libax;

#[no_mangle]
fn main() {
    match fork() {
        pid if pid > 0 => exit(0),
        0 => {
            http_server::main();
        }
        _ => {
            panic!("Error: fork()");
        }
    }
}

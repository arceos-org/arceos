#![no_std]
#![no_main]

use libax::process::wait;
use libax::{process::fork, task::yield_now};
use net_deamon::start_tcp;

#[macro_use]
extern crate libax;

use apps::http_client;
use apps::http_server;
use apps::shell;

fn fake_exec(f: fn()) {
    match fork() {
        pid if pid > 0 => {}
        0 => {
            f();
            loop {
                yield_now()
            }
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
    fake_exec(fs_deamon::init);
    fake_exec(http_client::main);
    fake_exec(http_server::main);
    //fake_exec(shell::main);

    loop {
        let mut ret: i32 = 0;
        let pid = wait(0, &mut ret);
        println!("Process {} exited with code {}", pid, ret);
    }
}

#![no_std]
#![no_main]

use libax::process::wait;
use microkernel_init::{fake_exec, real_exec};

#[macro_use]
extern crate libax;

#[no_mangle]
fn main() {
    println!("Hello world!");

    println!("Start TCP deamon");

    fake_exec(fs_deamon::init);
    #[cfg(feature = "net_deamon")]
    real_exec("./net_deamon");
    real_exec("./shell");

    loop {
        let mut ret: i32 = 0;
        let pid = wait(0, &mut ret);
        println!("init: process {} exited with code {}", pid, ret);
    }
}

#![no_std]
#![no_main]

use core::time::Duration;

use libax::process::wait;
use microkernel_init::fake_exec;

#[macro_use]
extern crate libax;

fn run_net_deamon() {
    net_deamon::init();
    net_deamon::start_tcp();
}

#[no_mangle]
fn main() {
    fake_exec(run_net_deamon);
    libax::task::sleep(Duration::from_millis(50));
    fake_exec(apps::http_client::main);
    let mut ret: i32 = 0;
    let pid = wait(0, &mut ret);
    println!("Process {} exited with code {}", pid, ret);
}

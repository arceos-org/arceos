#![no_std]
#![no_main]

use core::time::Duration;

use libax::process::wait;
use microkernel_init::fake_exec;

#[macro_use]
extern crate libax;

#[no_mangle]
fn main() {
    fake_exec(fs_deamon::init);
    libax::task::sleep(Duration::from_millis(50));
    fake_exec(apps::shell::main);
    let mut ret: i32 = 0;
    let pid = wait(0, &mut ret);
    println!("Process {} exited with code {}", pid, ret);
}

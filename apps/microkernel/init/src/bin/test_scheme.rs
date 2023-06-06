#![no_std]
#![no_main]

use libax::process::wait;
use microkernel_init::fake_exec;
use tests::scheme;

#[macro_use]
extern crate libax;

#[no_mangle]
fn main() {
    fake_exec(scheme::main);
    let mut ret: i32 = 0;
    let pid = wait(0, &mut ret);
    println!("Process {} exited with code {}", pid, ret);
}

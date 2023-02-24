#![no_std]
#![no_main]

use core::arch::asm;
use libax::println;

fn rasie_break_exception() {
    unsafe {
        #[cfg(target_arch = "x86_64")]
        asm!("int3");
        #[cfg(target_arch = "aarch64")]
        asm!("brk #0");
        #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
        asm!("ebreak");
    }
}

#[no_mangle]
fn main() {
    println!("Running exception tests...");
    rasie_break_exception();
    println!("Exception tests run OK!");
}

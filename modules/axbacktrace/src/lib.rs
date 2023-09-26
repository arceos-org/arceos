//! backtrace实现
//! 
//! NOTITE:
//! 开启本模块需要：
//! 1. 在根目录的 Makefile 中设置 `RUSTFLAGS ?= -Cforce-frame-pointers=yes`
//! 2. 在根目录的 Makefile 中，保证编译目标为 riscv64，因为 backtrace 目前只支持了该架构
//! 3. 在 app 的 Cargo.toml 设置加入名为 `axruntime/backtrace` 的 feature

#![no_std]

#[cfg(not(target_arch = "riscv64"))]
compile_error!("backtrace has only impl on riscv64");

#[macro_use]
extern crate log;

extern crate alter_trap;

use alter_trap::alter_trap_read_at;
use core::arch::asm;
use core::mem::size_of;

extern "C" {
    fn _stext();
    fn _etext();
}

pub fn backtrace() {
    unsafe {
        let mut ra: usize;
        asm!("mv {ptr}, ra", ptr = out(reg) ra);
        let mut fp: usize;
        asm!("mv {ptr}, fp", ptr = out(reg) fp);

        error!("stack backtrace:");
        unwind(ra, fp);
    }
}

fn unwind(mut ra: usize, mut fp: usize) -> Option<usize> {
    while ra >= _stext as usize && ra < _etext as usize {
        fp = alter_trap_read_at(fp - 16).as_option()?;
        ra = alter_trap_read_at(fp - 8).as_option()?;
        error!("ra={:#016X} fp={:#016X}", ra - size_of::<usize>(), fp);
    }
    Some(0)
}

#![no_std]

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

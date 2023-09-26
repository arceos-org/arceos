#![no_std]

#[macro_use]
extern crate log;

use core::arch::asm;
use core::mem::size_of;

pub fn backtrace() {
    extern "C" {
        fn _stext();
        fn _etext();
    }

    unsafe {
        let mut ra: usize;
        asm!("mv {ptr}, ra", ptr = out(reg) ra);
        let mut fp: usize;
        asm!("mv {ptr}, fp", ptr = out(reg) fp);

        error!("stack backtrace:");
        while ra >= _stext as usize && ra < _etext as usize && fp >= _stext as usize {
            fp = *((fp - 16) as *const usize);
            if fp < _stext as usize {
                break;
            }
            ra = *((fp - 8) as *const usize);
            error!("ra={:#016X} fp={:#016X}", ra - size_of::<usize>(), fp);
        }
    }
}

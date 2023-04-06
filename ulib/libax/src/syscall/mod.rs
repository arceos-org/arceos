mod entry;

#[macro_use]
pub mod logging;
pub use logging::__print_impl;
pub mod task;

use core::panic::PanicInfo;

//#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);
    task::exit(1);
}

/// Copied from rcore
pub fn syscall(id: usize, args: [usize; 6]) -> isize {
    let mut ret: isize;
    unsafe {
        core::arch::asm!("ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x13") args[3],
            in("x14") args[4],
            in("x15") args[5],
            in("x17") id
        );
    }
    ret
}

pub use logging::{debug, error, info, trace, warn};




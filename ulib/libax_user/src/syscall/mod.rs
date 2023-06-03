pub mod io;
pub mod process;
pub mod sync;
pub mod task;

use sys_number::SYS_TIME_NANO;
use syscall_number as sys_number;

/// Copied from rcore
#[allow(unused_variables)]
pub(crate) fn syscall(id: usize, args: [usize; 6]) -> isize {
    let mut ret: isize;
    #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
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
        ret
    }
    #[cfg(not(any(target_arch = "riscv64", target_arch = "riscv32")))]
    unimplemented!();
}

pub fn current_time_nanos() -> u64 {
    syscall(SYS_TIME_NANO, [0; 6]) as u64
}

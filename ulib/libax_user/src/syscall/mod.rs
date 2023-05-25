pub mod io;
pub mod task;
pub mod sync;


use sys_number::SYS_TIME_NANO;
use syscall_number as sys_number;

/// Copied from rcore
pub fn syscall(id: usize, args: [usize; 6]) -> isize {
    let mut ret: isize;
    unsafe {
        #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
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

pub fn current_time_nanos() -> u64 {
    syscall(SYS_TIME_NANO, [0; 6]) as u64
}

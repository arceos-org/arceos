pub mod io;
pub mod task;
pub mod sync;

pub mod sys_number {
    pub const SYS_WRITE: usize = 1;
    pub const SYS_EXIT: usize = 10;
    pub const SYS_SPAWN: usize = 11;
    pub const SYS_YIELD: usize = 12;
    pub const SYS_SLEEP: usize = 13;
    pub const SYS_SBRK: usize = 20;
    pub const SYS_FUTEX: usize = 30;
}

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

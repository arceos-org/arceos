//! syscalls about processes
use syscall_number::{SYS_FORK, SYS_WAIT};

use crate::syscall;

/// `fork` another process with the same memory contents and file tables.
pub fn fork() -> isize {
    syscall(SYS_FORK, [0, 0, 0, 0, 0, 0])
}

pub fn wait(pid: usize, ret: &mut i32) -> usize {
    syscall(SYS_WAIT, [pid, ret as *mut i32 as usize, 0, 0, 0, 0]) as usize
}

//! syscalls about processes
use syscall_number::SYS_FORK;

use crate::syscall;

/// `fork` another process with the same memory contents and file tables.
pub fn fork() -> isize {
    syscall(SYS_FORK, [0, 0, 0, 0, 0, 0])
}

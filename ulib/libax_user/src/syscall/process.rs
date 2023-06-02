use syscall_number::SYS_FORK;

use crate::syscall;

pub fn fork() -> isize {
    syscall(SYS_FORK, [0, 0, 0, 0, 0, 0])
}

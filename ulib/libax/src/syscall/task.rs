use crate::syscall;

use super::SYSCALL_EXIT;

pub fn exit(exit_code: i32) -> ! {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0, 0, 0, 0]);
    unreachable!()
}

use super::sys_number::SYS_EXIT;


pub fn exit(exitcode: usize) -> ! {
    crate::syscall(SYS_EXIT, [exitcode, 0, 0, 0, 0, 0]);
    unreachable!("program already terminated")
}

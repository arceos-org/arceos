use super::sys_number::SYS_WRITE;
pub fn write(fd: usize, s: &str) -> isize {
    crate::syscall(SYS_WRITE, [fd, s.as_ptr() as usize, s.len(), 0, 0, 0])
}

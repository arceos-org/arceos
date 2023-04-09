
use super::sys_number::SYS_WRITE;
pub fn write(s: &str) -> isize {
    crate::syscall(SYS_WRITE, [s.as_ptr() as usize, s.len(), 0, 0, 0, 0])
}

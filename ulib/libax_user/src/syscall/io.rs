use syscall_number::{io::OpenFlags, SYS_CLOSE, SYS_OPEN, SYS_READ, SYS_DUP};

use super::sys_number::SYS_WRITE;

pub fn write(fd: usize, s: &[u8]) -> isize {
    crate::syscall(SYS_WRITE, [fd, s.as_ptr() as usize, s.len(), 0, 0, 0])
}

pub fn read(fd: usize, s: &mut [u8]) -> isize {
    crate::syscall(SYS_READ, [fd, s.as_ptr() as usize, s.len(), 0, 0, 0])
}

pub fn open(path: &str, permission: OpenFlags) -> isize {
    crate::syscall(
        SYS_OPEN,
        [
            path.as_ptr() as usize,
            path.len(),
            permission.bits(),
            0,
            0,
            0,
        ],
    )
}

pub fn close(fd: usize) -> isize {
    crate::syscall(SYS_CLOSE, [fd, 0, 0, 0, 0, 0])
}
pub fn dup(fd: usize, buf: &str) -> isize {
    crate::syscall(SYS_DUP, [fd, buf.as_ptr() as usize, buf.len(), 0, 0, 0])
}

use scheme::Stat;
use syscall_number::{
    io::OpenFlags, SYS_CLOSE, SYS_DUP, SYS_FSTAT, SYS_FSYNC, SYS_LSEEK, SYS_OPEN, SYS_READ,
    SYS_RMDIR, SYS_UNLINK,
};

use super::sys_number::SYS_WRITE;

pub(crate) fn write(fd: usize, s: &[u8]) -> isize {
    crate::syscall(SYS_WRITE, [fd, s.as_ptr() as usize, s.len(), 0, 0, 0])
}

pub(crate) fn read(fd: usize, s: &mut [u8]) -> isize {
    crate::syscall(SYS_READ, [fd, s.as_ptr() as usize, s.len(), 0, 0, 0])
}

pub(crate) fn open(path: &str, permission: OpenFlags) -> isize {
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

pub(crate) fn close(fd: usize) -> isize {
    crate::syscall(SYS_CLOSE, [fd, 0, 0, 0, 0, 0])
}
pub(crate) fn dup(fd: usize, buf: &str) -> isize {
    crate::syscall(SYS_DUP, [fd, buf.as_ptr() as usize, buf.len(), 0, 0, 0])
}

pub(crate) fn lseek(fd: usize, offset: isize, whence: usize) -> isize {
    crate::syscall(SYS_LSEEK, [fd, offset as usize, whence as usize, 0, 0, 0])
}

pub(crate) fn remove_dir(path: &str) -> isize {
    crate::syscall(SYS_RMDIR, [path.as_ptr() as usize, path.len(), 0, 0, 0, 0])
}

pub(crate) fn remove_file(path: &str) -> isize {
    crate::syscall(SYS_UNLINK, [path.as_ptr() as usize, path.len(), 0, 0, 0, 0])
}

pub(crate) fn fstat(fd: usize, stat: *mut Stat) -> isize {
    crate::syscall(
        SYS_FSTAT,
        [fd, stat as usize, core::mem::size_of::<Stat>(), 0, 0, 0],
    )
}
pub(crate) fn fsync(fd: usize) -> isize {
    crate::syscall(SYS_FSYNC, [fd, 0, 0, 0, 0, 0])
}

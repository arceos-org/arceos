use arceos_posix_api::ctypes::iovec;
use log::info;

#[unsafe(no_mangle)]
pub fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
    info!("called sys_write");
    arceos_posix_api::sys_write(fd, buf as _, count)
}

#[unsafe(no_mangle)]
pub fn sys_writev(fd: i32, iov: *const iovec, iovcnt: usize) -> isize {
    info!("called sys_writev");
    unsafe { arceos_posix_api::sys_writev(fd, iov, iovcnt as _) }
}

#[cfg(feature = "fd")]
#[unsafe(no_mangle)]
pub fn sys_close(fd: i32) -> i32 {
    info!("called sys_close");
    arceos_posix_api::sys_close(fd) as _
}

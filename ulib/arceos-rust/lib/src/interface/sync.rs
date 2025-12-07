use arceos_api::modules::axlog::info;
use arceos_posix_api::ctypes::timespec;

#[unsafe(no_mangle)]
pub fn sys_futex_wait(
    address: *mut u32,
    expected: u32,
    timeout: *const timespec,
    flags: u32,
) -> i32 {
    // sys_futex_wait(address, expected, timeout, flags);
    // Placeholder implementation
    // info!("called sys_futex_wait");
    0
}

#[unsafe(no_mangle)]
pub fn sys_futex_wake(address: *mut u32, count: i32) -> i32 {
    // Placeholder implementation
    info!("called sys_futex_wake");
    0
}
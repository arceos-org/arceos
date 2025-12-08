use arceos_api::modules::axhal::time::wall_time_nanos;
use arceos_api::modules::axlog::info;
use arceos_posix_api::ctypes::{clockid_t, timespec};
use core::time::Duration;
use rand::prelude::SmallRng;
use rand::{RngCore, SeedableRng};

/// Fill `len` bytes in `buf` with cryptographically secure random data.
///
/// Returns either the number of bytes written to buf (a positive value) or
/// * `-EINVAL` if `flags` contains unknown flags.
/// * `-ENOSYS` if the system does not support random data generation.
#[unsafe(no_mangle)]
pub fn sys_read_entropy(buf: *mut u8, len: usize, _flags: u32) -> isize {
    // flags are currently ignored
    info!("called sys_read_entropy");
    let buffer = unsafe { core::slice::from_raw_parts_mut(buf, len) };
    let mut rng = SmallRng::seed_from_u64(wall_time_nanos());
    rng.fill_bytes(buffer);
    len as isize
}

#[unsafe(no_mangle)]
pub fn sys_clock_gettime(clockid: clockid_t, tp: *mut timespec) -> i32 {
    info!("called sys_clock_gettime with clockid {}, tp {:p}", clockid, tp);
    unsafe { arceos_posix_api::sys_clock_gettime(clockid, tp) }
}

/// suspend execution for microsecond intervals
///
/// The usleep() function suspends execution of the calling
/// thread for (at least) `usec` microseconds.
#[unsafe(no_mangle)]
pub fn sys_usleep(usec: u64) {
    info!("called sys_usleep with {} usec", usec);
    let duration = Duration::from_micros(usec);
    #[cfg(feature = "multitask")]
    arceos_api::modules::axtask::sleep(duration);
    #[cfg(not(feature = "multitask"))]
    arceos_api::modules::axhal::time::busy_wait(duration);
}

/// dummy implementation of futex wait
#[cfg(not(feature = "multitask"))]
#[unsafe(no_mangle)]
pub fn sys_futex_wait(
    address: *mut u32,
    expected: u32,
    timeout: *const timespec,
    flags: u32,
) -> i32 {
    0
}

/// dummy implementation of futex wake
#[cfg(not(feature = "multitask"))]
#[unsafe(no_mangle)]
pub fn sys_futex_wake(address: *mut u32, count: i32) -> i32 {
    0
}

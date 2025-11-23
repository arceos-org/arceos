use arceos_api::modules::axhal::time::wall_time_nanos;
use arceos_api::modules::axlog::{ax_println, info};
use arceos_api::sys::ax_terminate;
use arceos_posix_api::ctypes::timespec;
use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};

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

#[unsafe(no_mangle)]
pub fn sys_abort() -> ! {
    info!("called sys_abort");
    ax_terminate()
}

#[unsafe(no_mangle)]
pub fn sys_exit(code: i32) -> ! {
    info!("called sys_exit with code {}", code);
    ax_println!("[ArceOS] Process exited with code {}", code);
    ax_terminate()
}

#[unsafe(no_mangle)]
pub fn sys_read_entropy(buf: *mut u8, len: usize, flags: u32) -> isize {
    // TODO: flags are currently ignored
    info!("called sys_read_entropy");
    let buffer = unsafe { core::slice::from_raw_parts_mut(buf, len) };
    let mut rng = SmallRng::seed_from_u64(wall_time_nanos());
    rng.fill_bytes(buffer);
    len as isize
}

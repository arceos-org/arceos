use alloc::string::String;
use axhal::{cpu::this_cpu_id, time::current_time};
use axlog::ax_print;
use core::ffi::{c_int, c_uchar, c_uint};
use core::sync::atomic::{AtomicU64, Ordering::SeqCst};

#[no_mangle]
unsafe extern "C" fn lwip_print(str: *const c_uchar, mut args: ...) -> c_int {
    use printf_compat::{format, output};
    let mut s = String::new();
    let bytes_written = format(str, args.as_va_list(), output::fmt_write(&mut s));
    let now = current_time();
    let cpu_id = this_cpu_id();
    ax_print!(
        "[{:>3}.{:06} {}] {}",
        now.as_secs(),
        now.subsec_micros(),
        cpu_id,
        s
    );
    bytes_written
}

#[no_mangle]
extern "C" fn lwip_abort() {
    panic!("lwip_abort");
}

#[no_mangle]
extern "C" fn sys_now() -> c_uint {
    current_time().as_millis() as c_uint
}

static SEED: AtomicU64 = AtomicU64::new(0xa2ce_a2ce);

/// Sets the seed for the random number generator.
#[no_mangle]
extern "C" fn lwip_srand(seed: u32) {
    SEED.store(seed.wrapping_sub(1) as u64, SeqCst);
}

/// Returns a 32-bit unsigned pseudo random interger.
#[no_mangle]
extern "C" fn lwip_rand_u32() -> u32 {
    let new_seed = SEED.load(SeqCst).wrapping_mul(6364136223846793005) + 1;
    SEED.store(new_seed, SeqCst);
    (new_seed >> 33) as u32
}

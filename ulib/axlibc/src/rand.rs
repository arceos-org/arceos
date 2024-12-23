//! Random number generator.

use core::{
    ffi::{c_int, c_long, c_uint},
    sync::atomic::{AtomicU64, Ordering::SeqCst},
};

static SEED: AtomicU64 = AtomicU64::new(0xa2ce_a2ce);

/// Sets the seed for the random number generator.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn srand(seed: c_uint) {
    SEED.store(seed.wrapping_sub(1) as u64, SeqCst);
}

/// Returns a 32-bit unsigned pseudo random interger.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rand() -> c_int {
    let new_seed = SEED.load(SeqCst).wrapping_mul(6364136223846793005) + 1;
    SEED.store(new_seed, SeqCst);
    (new_seed >> 33) as c_int
}

/// Returns a 64-bit unsigned pseudo random number.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn random() -> c_long {
    let new_seed = SEED.load(SeqCst).wrapping_mul(6364136223846793005) + 1;
    SEED.store(new_seed, SeqCst);
    new_seed as c_long
}

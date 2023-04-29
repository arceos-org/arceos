use core::sync::atomic::{AtomicU64, Ordering::SeqCst};

static SEED: AtomicU64 = AtomicU64::new(0xa2ce_a2ce);

pub fn srand(seed: u32) {
    SEED.store(seed.wrapping_sub(1) as u64, SeqCst);
}

pub fn rand_u32() -> u32 {
    let new_seed = SEED.load(SeqCst).wrapping_mul(6364136223846793005) + 1;
    SEED.store(new_seed, SeqCst);
    (new_seed >> 33) as u32
}

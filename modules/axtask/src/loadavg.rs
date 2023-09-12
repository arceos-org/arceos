#[cfg(feature = "irq")]
use core::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "irq")]
const FSHIFT: u64 = 11;
#[cfg(feature = "irq")]
const FIXED_1: u64 = 1 << FSHIFT;
#[cfg(feature = "irq")]
const LOAD_FREQ: u64 = 5 * axhal::time::NANOS_PER_SEC + 1;

#[cfg(feature = "irq")]
/* 1/exp(5sec/1min) as fixed-point */
/* 1/exp(5sec/5min) */
/* 1/exp(5sec/15min) */
const EXP: [u64; 3] = [1884, 2014, 2037];

#[cfg(feature = "irq")]
static mut IDLE_CNT: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "irq")]
static mut ALL_CNT: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "irq")]
static mut LAST_UPDATE: AtomicU64 = AtomicU64::new(0);
// TODO: if irq is disabled, what value should AVENRUN be?
static mut AVENRUN: [u64; 3] = [0, 0, 0];

pub fn get_avenrun(loads: &mut [u64; 3]) {
    for i in 0..3 {
        unsafe {
            // TODO: disable irq for safety
            loads[i] = AVENRUN[i];
        }
    }
}

#[cfg(feature = "irq")]
/*
 * a1 = a0 * e + a * (1 - e)
 */
fn calc_load(load: u64, exp: u64, active: u64) -> u64 {
    let mut newload: u64 = load * exp + active * (FIXED_1 - exp);
    if active >= load {
        newload += FIXED_1 - 1;
    }
    return newload / FIXED_1;
}

#[cfg(feature = "irq")]
/*
 * calc_load - update the avenrun load estimates 10 ticks after the
 * CPUs have updated calc_load_tasks.
 *
 * Called from the global timer code.
 */
pub(crate) fn calc_load_tick(is_idle: bool) {
    if is_idle {
        unsafe {
            IDLE_CNT.fetch_add(1, Ordering::Relaxed);
        }
    }
    unsafe {
        ALL_CNT.fetch_add(1, Ordering::Relaxed);
    }

    let curr = axhal::time::current_time_nanos();

    if curr - unsafe { LAST_UPDATE.load(Ordering::Relaxed) } < LOAD_FREQ {
        return;
    }
    let idle_cnt;
    let all_cnt;
    unsafe {
        LAST_UPDATE.store(curr, Ordering::Relaxed);
        idle_cnt = IDLE_CNT.load(Ordering::Relaxed);
        IDLE_CNT.store(0, Ordering::Relaxed);
        all_cnt = ALL_CNT.load(Ordering::Relaxed);
        ALL_CNT.store(0, Ordering::Relaxed);
    }
    for i in 0..3 {
        unsafe {
            AVENRUN[i] = calc_load(AVENRUN[i], EXP[i], (all_cnt - idle_cnt) * FIXED_1 / all_cnt);
        }
    }
}

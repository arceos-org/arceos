use aarch64_cpu::registers::{CNTFRQ_EL0, CNTPCT_EL0};
use lazy_init::LazyInit;
use ratio::Ratio;
use tock_registers::interfaces::Readable;

use crate::time::NANOS_PER_SEC;

#[allow(unused)]
const PHYS_TIMER_IRQ_NUM: usize = 30;

static CNTPCT_TO_NANOS_RATIO: LazyInit<Ratio> = LazyInit::new();

pub fn current_ticks() -> u64 {
    CNTPCT_EL0.get()
}

#[inline(never)]
pub fn ticks_to_nanos(ticks: u64) -> u64 {
    CNTPCT_TO_NANOS_RATIO.mul_trunc(ticks)
}

pub(crate) fn init() {
    CNTPCT_TO_NANOS_RATIO.init_by(Ratio::new(NANOS_PER_SEC as u32, CNTFRQ_EL0.get() as u32));
}

use aarch64_cpu::registers::{CNTFRQ_EL0, CNTPCT_EL0, CNTP_CTL_EL0, CNTP_TVAL_EL0};
use lazy_init::LazyInit;
use ratio::Ratio;
use tock_registers::interfaces::{Readable, Writeable};

/// The timer IRQ number.
pub const TIMER_IRQ_NUM: usize = 30; // physical timer, type=PPI, id=14

static CNTPCT_TO_NANOS_RATIO: LazyInit<Ratio> = LazyInit::new();
static NANOS_TO_CNTPCT_RATIO: LazyInit<Ratio> = LazyInit::new();

/// Returns the current clock time in hardware ticks.
#[inline]
pub fn current_ticks() -> u64 {
    CNTPCT_EL0.get()
}

/// Converts hardware ticks to nanoseconds.
#[inline]
pub fn ticks_to_nanos(ticks: u64) -> u64 {
    CNTPCT_TO_NANOS_RATIO.mul_trunc(ticks)
}

/// Converts nanoseconds to hardware ticks.
#[inline]
pub fn nanos_to_ticks(nanos: u64) -> u64 {
    NANOS_TO_CNTPCT_RATIO.mul_trunc(nanos)
}

/// Set a one-shot timer.
///
/// A timer interrupt will be triggered at the given deadline (in nanoseconds).
pub fn set_oneshot_timer(deadline_ns: u64) {
    let cnptct = CNTPCT_EL0.get();
    let cnptct_deadline = nanos_to_ticks(deadline_ns);
    if cnptct < cnptct_deadline {
        let interval = cnptct_deadline - cnptct;
        debug_assert!(interval <= u32::MAX as u64);
        CNTP_TVAL_EL0.set(interval);
    } else {
        CNTP_TVAL_EL0.set(0);
    }
}

pub(super) fn init() {
    let freq = CNTFRQ_EL0.get();
    CNTPCT_TO_NANOS_RATIO.init_by(Ratio::new(crate::time::NANOS_PER_SEC as u32, freq as u32));
    NANOS_TO_CNTPCT_RATIO.init_by(CNTPCT_TO_NANOS_RATIO.inverse());
    CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET);
    CNTP_TVAL_EL0.set(0);
    super::irq::set_enable(TIMER_IRQ_NUM, true);
}

#[cfg(feature = "smp")]
pub(super) fn init_secondary() {
    CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET);
    CNTP_TVAL_EL0.set(0);
    super::irq::set_enable(TIMER_IRQ_NUM, true);
}

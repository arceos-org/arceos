use loongarch64::time::Time;

const NANOS_PER_TICK: u64 = crate::time::NANOS_PER_SEC / axconfig::TIMER_FREQUENCY as u64;

/// Returns the current clock time in hardware ticks.
#[inline]
pub fn current_ticks() -> u64 {
    Time::read() as u64
}

/// Converts hardware ticks to nanoseconds.
#[inline]
pub const fn ticks_to_nanos(ticks: u64) -> u64 {
    ticks * NANOS_PER_TICK
}

/// Converts nanoseconds to hardware ticks.
#[inline]
pub const fn nanos_to_ticks(nanos: u64) -> u64 {
    nanos / NANOS_PER_TICK
}

/// Set a one-shot timer.
///
/// A timer interrupt will be triggered at the given deadline (in nanoseconds).
#[cfg(feature = "irq")]
pub fn set_oneshot_timer(deadline_ns: u64) {
    use loongarch64::register::tcfg;
    tcfg::set_en(true);
    tcfg::set_periodic(false);
    tcfg::set_init_val(nanos_to_ticks(deadline_ns) as usize);
}
#[cfg(feature = "irq")]
pub(super) fn init_primary() {
    {
        use crate::arch::disable_irqs;
        disable_irqs();
        // 清除时钟中断
        ticlr::clear_timer_interrupt();
    }
}

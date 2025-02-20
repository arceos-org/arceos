use loongArch64::time::Time;

const NANOS_PER_TICK: u64 = crate::time::NANOS_PER_SEC / axconfig::devices::TIMER_FREQUENCY as u64;

/// RTC wall time offset in nanoseconds at monotonic time base.
static mut RTC_EPOCHOFFSET_NANOS: u64 = 0;

/// Returns the current clock time in hardware ticks.
#[inline]
pub fn current_ticks() -> u64 {
    Time::read() as _
}

/// Return epoch offset in nanoseconds (wall time offset to monotonic clock start).
#[inline]
pub fn epochoffset_nanos() -> u64 {
    unsafe { RTC_EPOCHOFFSET_NANOS }
}

/// Converts hardware ticks to nanoseconds.
#[inline]
pub fn ticks_to_nanos(ticks: u64) -> u64 {
    ticks * NANOS_PER_TICK
}

/// Converts nanoseconds to hardware ticks.
#[inline]
pub fn nanos_to_ticks(nanos: u64) -> u64 {
    nanos / NANOS_PER_TICK
}

/// Set a one-shot timer.
///
/// A timer interrupt will be triggered at the specified monotonic time deadline (in nanoseconds).
///
/// LoongArch64 TCFG CSR: <https://loongson.github.io/LoongArch-Documentation/LoongArch-Vol1-EN.html#timer-configuration>
#[cfg(feature = "irq")]
pub fn set_oneshot_timer(deadline_ns: u64) {
    use loongArch64::register::tcfg;

    let ticks_now = current_ticks();
    let ticks_deadline = nanos_to_ticks(deadline_ns);
    let init_value = ticks_deadline - ticks_now;

    // This initial value must be an integer multiple of 4.
    tcfg::set_init_val(((init_value + 3) & !3) as _);
    tcfg::set_periodic(false);
    tcfg::set_en(true);
}

pub(super) fn init_percpu() {
    use loongArch64::register::tcfg;

    tcfg::set_init_val(0);
    tcfg::set_periodic(false);
    tcfg::set_en(true);

    #[cfg(feature = "irq")]
    {
        super::irq::set_enable(super::irq::TIMER_IRQ_NUM, true);
    }
}

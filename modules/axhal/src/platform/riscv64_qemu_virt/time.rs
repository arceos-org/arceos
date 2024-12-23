use riscv::register::time;

const NANOS_PER_TICK: u64 = crate::time::NANOS_PER_SEC / axconfig::devices::TIMER_FREQUENCY as u64;
/// RTC wall time offset in nanoseconds at monotonic time base.
static mut RTC_EPOCHOFFSET_NANOS: u64 = 0;

/// Returns the current clock time in hardware ticks.
#[inline]
pub fn current_ticks() -> u64 {
    time::read() as u64
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

/// Return epoch offset in nanoseconds (wall time offset to monotonic clock start).
pub fn epochoffset_nanos() -> u64 {
    unsafe { RTC_EPOCHOFFSET_NANOS }
}

/// Set a one-shot timer.
///
/// A timer interrupt will be triggered at the specified monotonic time deadline (in nanoseconds).
#[cfg(feature = "irq")]
pub fn set_oneshot_timer(deadline_ns: u64) {
    sbi_rt::set_timer(nanos_to_ticks(deadline_ns));
}

pub(super) fn init_early() {
    #[cfg(feature = "rtc")]
    if axconfig::devices::RTC_PADDR != 0 {
        use crate::mem::phys_to_virt;
        use memory_addr::PhysAddr;
        use riscv_goldfish::Rtc;

        const GOLDFISH_BASE: PhysAddr = pa!(axconfig::devices::RTC_PADDR);
        // Get the current time in microseconds since the epoch (1970-01-01) from the riscv RTC.
        // Subtract the timer ticks to get the actual time when ArceOS was booted.
        let epoch_time_nanos =
            Rtc::new(phys_to_virt(GOLDFISH_BASE).as_usize()).get_unix_timestamp() * 1_000_000_000;

        unsafe {
            RTC_EPOCHOFFSET_NANOS = epoch_time_nanos - ticks_to_nanos(current_ticks());
        }
    }
}

pub(super) fn init_percpu() {
    #[cfg(feature = "irq")]
    sbi_rt::set_timer(0);
}

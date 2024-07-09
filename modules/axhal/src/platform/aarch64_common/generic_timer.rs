#![allow(unused_imports)]

use aarch64_cpu::registers::{CNTFRQ_EL0, CNTPCT_EL0, CNTP_CTL_EL0, CNTP_TVAL_EL0};
use ratio::Ratio;
use tock_registers::interfaces::{Readable, Writeable};

use memory_addr::{PhysAddr, VirtAddr};
use rtc::Rtc;
use spinlock::SpinNoIrq;

use crate::mem::phys_to_virt;

static mut CNTPCT_TO_NANOS_RATIO: Ratio = Ratio::zero();
static mut NANOS_TO_CNTPCT_RATIO: Ratio = Ratio::zero();

static mut INIT_TICK: u64 = 0;

/// Returns the current clock time in hardware ticks.
#[inline]
pub fn current_ticks() -> u64 {
    CNTPCT_EL0.get() + unsafe { INIT_TICK }
}

/// Converts hardware ticks to nanoseconds.
#[inline]
pub fn ticks_to_nanos(ticks: u64) -> u64 {
    unsafe { CNTPCT_TO_NANOS_RATIO.mul_trunc(ticks) }
}

/// Converts nanoseconds to hardware ticks.
#[inline]
pub fn nanos_to_ticks(nanos: u64) -> u64 {
    unsafe { NANOS_TO_CNTPCT_RATIO.mul_trunc(nanos) }
}

/// Set a one-shot timer.
///
/// A timer interrupt will be triggered at the given deadline (in nanoseconds).
#[cfg(feature = "irq")]
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

/// Early stage initialization: stores the timer frequency.
pub(crate) fn init_early() {
    let freq = CNTFRQ_EL0.get();
    unsafe {
        CNTPCT_TO_NANOS_RATIO = Ratio::new(crate::time::NANOS_PER_SEC as u32, freq as u32);
        NANOS_TO_CNTPCT_RATIO = CNTPCT_TO_NANOS_RATIO.inverse();
    }
}

pub(crate) fn init_rtc() {
    // Init system Real Time Clock (RTC).

    const PL031_BASE: PhysAddr = PhysAddr::from(axconfig::RTC_PADDR);

    static RTC_LOCK: SpinNoIrq<()> = SpinNoIrq::new(());

    let _guard = RTC_LOCK.lock();
    // Get the current time in microseconds since the epoch (1970-01-01) from the aarch64 RTC.
    // Subtract the timer ticks to get the actual time when ArceOS was booted.
    let current_time_nanos =
        Rtc::new(phys_to_virt(PL031_BASE).as_usize()).get_unix_timestamp() * 1_000_000_000;
    let current_ticks = nanos_to_ticks(current_time_nanos);

    unsafe { INIT_TICK = current_ticks - CNTPCT_EL0.get() };
}

pub(crate) fn init_percpu() {
    #[cfg(feature = "irq")]
    {
        CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET);
        CNTP_TVAL_EL0.set(0);
        crate::platform::irq::set_enable(crate::platform::irq::TIMER_IRQ_NUM, true);
    }
}

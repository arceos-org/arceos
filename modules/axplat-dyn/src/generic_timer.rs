//! ARM Generic Timer.

struct GenericTimer;

#[impl_plat_interface]
impl axplat::time::TimeIf for GenericTimer {
    /// Returns the current clock time in hardware ticks.
    fn current_ticks() -> u64 {
        somehal::timer::ticks() as _
    }

    /// Converts hardware ticks to nanoseconds.
    fn ticks_to_nanos(ticks: u64) -> u64 {
        let freq = somehal::timer::freq() as u64;
        if freq == 0 {
            return 0;
        }
        (ticks * axplat::time::NANOS_PER_SEC) / freq
    }

    /// Converts nanoseconds to hardware ticks.
    fn nanos_to_ticks(nanos: u64) -> u64 {
        let freq = somehal::timer::freq() as u64;
        if freq == 0 {
            return 0;
        }
        (nanos * freq) / axplat::time::NANOS_PER_SEC
    }

    /// Return epoch offset in nanoseconds (wall time offset to monotonic
    /// clock start).
    fn epochoffset_nanos() -> u64 {
        0
    }
    /// Returns the IRQ number for the timer interrupt.
    #[cfg(feature = "irq")]
    fn irq_num() -> usize {
        somehal::irq::systick_irq().into()
    }
    /// Set a one-shot timer.
    ///
    /// A timer interrupt will be triggered at the specified monotonic time
    /// deadline (in nanoseconds).
    #[cfg(feature = "irq")]
    fn set_oneshot_timer(deadline_ns: u64) {
        let cnptct = somehal::timer::ticks() as u64;
        let deadline = GenericTimer::nanos_to_ticks(deadline_ns);
        let interval = if cnptct < deadline {
            let interval = deadline - cnptct;
            debug_assert!(interval <= u32::MAX as u64);
            interval
        } else {
            0
        };

        somehal::timer::set_next_event_in_ticks(interval as _);
    }
}

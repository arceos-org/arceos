/// Returns the current clock time in hardware ticks.
pub fn current_ticks() -> u64 {
    0
}

/// Converts hardware ticks to nanoseconds.
pub fn ticks_to_nanos(ticks: u64) -> u64 {
    ticks
}

/// Converts nanoseconds to hardware ticks.
pub fn nanos_to_ticks(nanos: u64) -> u64 {
    nanos
}

/// Set a one-shot timer.
///
/// A timer interrupt will be triggered at the given deadline (in nanoseconds).
pub fn set_oneshot_timer(deadline_ns: u64) {}

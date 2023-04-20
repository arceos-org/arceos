//! Time-related operations.

/// A measurement of the system clock.
///
/// Currently, it reuses the [`core::time::Duration`] type. But it does not
/// represent a duration, but a clock time.
pub type TimeValue = core::time::Duration;

pub use crate::platform::time::{
    current_ticks, nanos_to_ticks, set_oneshot_timer, ticks_to_nanos, TIMER_IRQ_NUM,
};

/// Number of milliseconds in a second.
pub const MILLIS_PER_SEC: u64 = 1_000;
/// Number of microseconds in a second.
pub const MICROS_PER_SEC: u64 = 1_000_000;
/// Number of nanoseconds in a second.
pub const NANOS_PER_SEC: u64 = 1_000_000_000;
/// Number of nanoseconds in a millisecond.
pub const NANOS_PER_MILLIS: u64 = 1_000_000;
/// Number of nanoseconds in a microsecond.
pub const NANOS_PER_MICROS: u64 = 1_000;

/// Returns the current clock time in nanoseconds.
pub fn current_time_nanos() -> u64 {
    ticks_to_nanos(current_ticks())
}

/// Returns the current clock time in [`TimeValue`].
pub fn current_time() -> TimeValue {
    TimeValue::from_nanos(current_time_nanos())
}

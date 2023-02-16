pub type TimeValue = core::time::Duration;

pub use crate::platform::time::{current_ticks, ticks_to_nanos};

pub const MILLIS_PER_SEC: u64 = 1_000;
pub const MICROS_PER_SEC: u64 = 1_000_000;
pub const NANOS_PER_SEC: u64 = 1_000_000_000;
pub const NANOS_PER_MILLIS: u64 = 1_000_000;
pub const NANOS_PER_MICROS: u64 = 1_000;

pub fn current_time_nanos() -> u64 {
    ticks_to_nanos(current_ticks())
}

pub fn current_time() -> TimeValue {
    TimeValue::from_nanos(current_time_nanos())
}

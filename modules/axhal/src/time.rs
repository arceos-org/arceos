//! Time-related operations.

pub use axplat::time::{
    Duration, MICROS_PER_SEC, MILLIS_PER_SEC, NANOS_PER_MICROS, NANOS_PER_MILLIS, NANOS_PER_SEC,
    TimeValue, busy_wait, busy_wait_until, current_ticks, epochoffset_nanos, monotonic_time,
    monotonic_time_nanos, nanos_to_ticks, ticks_to_nanos, wall_time, wall_time_nanos,
};
#[cfg(feature = "irq")]
pub use axplat::time::{irq_num, set_oneshot_timer};

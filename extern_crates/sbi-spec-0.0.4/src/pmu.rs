//! Chapter 11. Performance Monitoring Unit Extension (EID #0x504D55 "PMU")

/// Extension ID for Performance Monitoring Unit extension
pub const EID_PMU: usize = crate::eid_from_str("PMU") as _;
pub use fid::*;

/// Declared in §11.11
mod fid {
    /// Function ID to get the number of counters, both hardware and firmware
    ///
    /// Declared in §11.5
    pub const PMU_NUM_COUNTERS: usize = 0;
    /// Function ID to get details about the specified counter
    ///
    /// Declared in §11.6
    pub const PMU_COUNTER_GET_INFO: usize = 1;
    /// Function ID to find and configure a counter from a set of counters
    ///
    /// Declared in §11.7
    pub const PMU_COUNTER_CONFIG_MATCHING: usize = 2;
    /// Function ID to start or enable a set of counters on the calling hart with the specified initial value
    ///
    /// Declared in §11.8
    pub const PMU_COUNTER_START: usize = 3;
    /// Function ID to stop or disable a set of counters on the calling hart
    ///
    /// Declared in §11.9
    pub const PMU_COUNTER_STOP: usize = 4;
    /// Function ID to provide the current value of a firmware counter
    ///
    /// Declared in §11.10
    pub const PMU_COUNTER_FW_READ: usize = 5;
}

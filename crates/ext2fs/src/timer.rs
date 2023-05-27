/// Get time from it
pub trait TimeProvider {
    /// Get current time since epoch
    fn get_current_time(&self) -> u32;
}

/// Always retuen zero
pub struct ZeroTimeProvider;

impl TimeProvider for ZeroTimeProvider {
    fn get_current_time(&self) -> u32 {
        0
    }
}

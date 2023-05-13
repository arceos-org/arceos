pub trait TimeProvider {
    fn get_current_time(&self) -> u32;
}

pub struct ZeroTimeProvider;

impl TimeProvider for ZeroTimeProvider {
    fn get_current_time(&self) -> u32 {
        0
    }
}
//use log::info;
use crate_interface::call_interface;
use crate_interface::def_interface;

struct LogMyTimeImpl;

#[crate_interface::impl_interface]
impl LogMyTime for LogMyTimeImpl {
    fn current_time() -> core::time::Duration {
        axhal::time::current_time()
    }
}

#[cfg(not(feature = "std"))]
#[def_interface]
pub trait LogMyTime {
    /// get current time
    fn current_time() -> core::time::Duration;
}

/// get current time
pub fn current_ticks() -> isize {
    let mytime = call_interface!(LogMyTime::current_time).as_nanos();
    mytime as isize
}

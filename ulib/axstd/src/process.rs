//! A module for working with processes.
//!
//! Since ArceOS is a unikernel, there is no concept of processes. The
//! process-related functions will affect the entire system, such as [`exit`]
//! will shutdown the whole system.

/// Shutdown the whole system.
pub fn exit(_exit_code: i32) -> ! {
    arceos_api::sys::ax_terminate();
}

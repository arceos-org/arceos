#![allow(missing_docs)]

#[cfg(not(feature = "multitask"))]
use core::sync::atomic::AtomicU32;
#[cfg(not(feature = "multitask"))]
use core::time::Duration;

#[cfg(feature = "multitask")]
mod task;

#[cfg(feature = "fs")]
mod fs;

//
// Just single task, i.e., NO 'multitask' feature
//
#[cfg(not(feature = "multitask"))]
#[no_mangle]
pub fn sys_futex_wait(_: &AtomicU32, _: u32, _: Option<Duration>) -> bool {
    true
}

#[cfg(not(feature = "multitask"))]
#[no_mangle]
pub fn sys_futex_wake(_: &AtomicU32, _: i32) {}

#[cfg(all(feature = "alloc", not(feature = "fs")))]
#[no_mangle]
pub fn sys_getcwd() -> Result<alloc::string::String, axerrno::AxError> {
    Err(axerrno::AxError::NotFound)
}

#[cfg(feature = "multitask")]
pub use axsync::Mutex;

#[cfg(feature = "multitask")]
pub use axtask::WaitQueue;

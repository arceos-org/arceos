#[cfg(not(feature = "multitask"))]
mod no_thread;
#[cfg(not(feature = "multitask"))]
pub use no_thread::Condvar;

#[cfg(feature = "multitask")]
mod multitask;
#[cfg(feature = "multitask")]
pub use multitask::Condvar;

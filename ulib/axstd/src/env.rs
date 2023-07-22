//! Inspection and manipulation of the process’s environment.

#[cfg(feature = "fs")]
pub use axfs::api::{current_dir, set_current_dir};

//! Inspection and manipulation of the process’s environment.

#[cfg(feature = "fs")]
extern crate alloc;

#[cfg(feature = "fs")]
use {crate::io, alloc::string::String};

/// Returns the current working directory as a [`String`].
#[cfg(feature = "fs")]
pub fn current_dir() -> io::Result<String> {
    axfs::api::current_dir()
}

/// Changes the current working directory to the specified path.
#[cfg(feature = "fs")]
pub fn set_current_dir(path: &str) -> io::Result<()> {
    axfs::api::set_current_dir(path)
}

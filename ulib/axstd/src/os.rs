//! OS-specific functionality.

/// ArceOS-specific definitions.
pub mod arceos {
    pub use arceos_api as api;
    #[cfg(feature = "fs")]
    pub use axfs;
}

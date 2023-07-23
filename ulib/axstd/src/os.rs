//! OS-specific functionality.

/// ArceOS-specific definitions.
pub mod arceos {
    #[cfg(feature = "display")]
    pub use axdisplay;
    #[cfg(feature = "fs")]
    pub use axfs;
    #[cfg(feature = "multitask")]
    pub use axtask;
}

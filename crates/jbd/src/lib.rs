//! Crate for the journaling block device layer (like Linux).
//! Supports single-threaded journaling operations.

#![no_std]

/// Checkpointing routines.
pub mod checkpoint;
/// Commit routines.
pub mod commit;
mod config;
mod disk;
/// Error types.
pub mod err;
/// Journaling structure and routines.
pub mod journal;
/// Recovery routines.
mod recovery;
/// Revoke routines.
pub mod revoke;
/// The adaptation layer.
pub mod sal;
mod tx;
mod util;

pub use crate::journal::Journal;
pub use crate::tx::Handle;

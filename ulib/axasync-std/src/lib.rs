//! # The ArceOS Async Standard Library
//!
//! [ArceOS]: https://github.com/arceos-org/arceos

#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]

#[macro_use]
mod macros;
pub mod task;

/// Traits, helpers, and type definitions for core I/O functionality.
pub mod io {

    /// A specialized [`Result`] type for I/O operations.
    ///
    /// This type is broadly used across [`axasync_std::io`] for any operation which may
    /// produce an error.
    ///
    /// This typedef is generally used to avoid writing out [`io::Error`] directly and
    /// is otherwise a direct mapping to [`Result`].
    ///
    /// While usual Rust style is to import types directly, aliases of [`Result`]
    /// often are not, to make it easier to distinguish between them. [`Result`] is
    /// generally assumed to be [`std::result::Result`][`Result`], and so users of this alias
    /// will generally use `io::Result` instead of shadowing the [prelude]'s import
    /// of [`std::result::Result`][`Result`].
    ///
    /// [`axstd::io`]: crate::io
    /// [`io::Error`]: Error
    pub type Result<T> = axio::Result<T>;
}

//! Traits, helpers, and type definitions for core I/O functionality.

mod stdio;

pub use axio::prelude;
pub use axio::{BufRead, BufReader, Error, Read, Seek, SeekFrom, Write};

#[doc(hidden)]
pub use self::stdio::__print_impl;
pub use self::stdio::{stdin, stdout, Stdin, StdinLock, Stdout, StdoutLock};

/// A specialized [`Result`] type for I/O operations.
///
/// This type is broadly used across [`axstd::io`] for any operation which may
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

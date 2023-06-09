//! Error code definition used by [ArceOS](https://github.com/rcore-os/arceos).
//!
//! It provides two error types and the corresponding result types:
//!
//! - [`AxError`] and [`AxResult`]: A generic error type similar to
//!   [`std::io::ErrorKind`].
//! - [`LinuxError`] and [`LinuxResult`]: Linux specific error codes defined in
//!   `errno.h`. It can be converted from [`AxError`].
//!
//! [`std::io::ErrorKind`]: https://doc.rust-lang.org/std/io/enum.ErrorKind.html

#![no_std]

mod linux_errno {
    include!(concat!(env!("OUT_DIR"), "/linux_errno.rs"));
}

pub use linux_errno::LinuxError;

/// The error type used by ArceOS.
///
/// Similar to [`std::io::ErrorKind`].
///
/// [`std::io::ErrorKind`]: https://doc.rust-lang.org/std/io/enum.ErrorKind.html
#[repr(i32)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxError {
    /// A socket address could not be bound because the address is already in use elsewhere.
    AddrInUse,
    /// An entity already exists, often a file.
    AlreadyExists,
    /// Bad address.
    BadAddress,
    /// Bad internal state.
    BadState,
    /// The connection was refused by the remote server,
    ConnectionRefused,
    /// A non-empty directory was specified where an empty directory was expected.
    DirectoryNotEmpty,
    /// Data not valid for the operation were encountered.
    ///
    /// Unlike [`InvalidInput`], this typically means that the operation
    /// parameters were valid, however the error was caused by malformed
    /// input data.
    ///
    /// For example, a function that reads a file into a string will error with
    /// `InvalidData` if the file's contents are not valid UTF-8.
    ///
    /// [`InvalidInput`]: AxError::InvalidInput
    InvalidData,
    /// Invalid parameter/argument.
    InvalidInput,
    /// Input/output error.
    Io,
    /// The filesystem object is, unexpectedly, a directory.
    IsADirectory,
    /// Not enough space/cannot allocate memory.
    NoMemory,
    /// A filesystem object is, unexpectedly, not a directory.
    NotADirectory,
    /// The network operation failed because it was not connected yet.
    NotConnected,
    /// The requested entity is not found.
    NotFound,
    /// The operation lacked the necessary privileges to complete.
    PermissionDenied,
    /// Device or resource is busy.
    ResourceBusy,
    /// The underlying storage (typically, a filesystem) is full.
    StorageFull,
    /// An error returned when an operation could not be completed because an
    /// "end of file" was reached prematurely.
    UnexpectedEof,
    /// This operation is unsupported or unimplemented.
    Unsupported,
    /// The operation needs to block to complete, but the blocking operation was
    /// requested to not occur.
    WouldBlock,
    /// An error returned when an operation could not be completed because a
    /// call to `write()` returned [`Ok(0)`](Ok).
    WriteZero,
}

/// A specialized [`Result`] type with [`AxError`] as the error type.
pub type AxResult<T = ()> = Result<T, AxError>;

/// A specialized [`Result`] type with [`LinuxError`] as the error type.
pub type LinuxResult<T = ()> = Result<T, LinuxError>;

/// Convenience method to construct an [`AxError`] type while printing a warning
/// message.
///
/// # Examples
///
/// ```
/// # use axerrno::{ax_err_type, AxError};
/// #
/// // Also print "[AxError::AlreadyExists]" if the `log` crate is enabled.
/// assert_eq!(
///     ax_err_type!(AlreadyExists),
///     AxError::AlreadyExists,
/// );
///
/// // Also print "[AxError::BadAddress] the address is 0!" if the `log` crate
/// // is enabled.
/// assert_eq!(
///     ax_err_type!(BadAddress, "the address is 0!"),
///     AxError::BadAddress,
/// );
/// ```
#[macro_export]
macro_rules! ax_err_type {
    ($err: ident) => {{
        use $crate::AxError::*;
        $crate::__priv::warn!("[AxError::{:?}]", $err);
        $err
    }};
    ($err: ident, $msg: expr) => {{
        use $crate::AxError::*;
        $crate::__priv::warn!("[AxError::{:?}] {}", $err, $msg);
        $err
    }};
}

/// Ensure a condition is true. If it is not, return from the function
/// with an error.
///
/// ## Examples
///
/// ```rust
/// # use axerrno::{ensure, ax_err, AxError, AxResult};
///
/// fn example(user_id: i32) -> AxResult {
///     ensure!(user_id > 0, ax_err!(InvalidInput));
///     // After this point, we know that `user_id` is positive.
///     let user_id = user_id as u32;
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! ensure {
    ($predicate:expr, $context_selector:expr $(,)?) => {
        if !$predicate {
            return $context_selector;
        }
    };
}

/// Convenience method to construct an [`Err(AxError)`] type while printing a
/// warning message.
///
/// # Examples
///
/// ```
/// # use axerrno::{ax_err, AxResult, AxError};
/// #
/// // Also print "[AxError::AlreadyExists]" if the `log` crate is enabled.
/// assert_eq!(
///     ax_err!(AlreadyExists),
///     AxResult::<()>::Err(AxError::AlreadyExists),
/// );
///
/// // Also print "[AxError::BadAddress] the address is 0!" if the `log` crate is enabled.
/// assert_eq!(
///     ax_err!(BadAddress, "the address is 0!"),
///     AxResult::<()>::Err(AxError::BadAddress),
/// );
/// ```
/// [`Err(AxError)`]: Err
#[macro_export]
macro_rules! ax_err {
    ($err: ident) => {
        Err($crate::ax_err_type!($err))
    };
    ($err: ident, $msg: expr) => {
        Err($crate::ax_err_type!($err, $msg))
    };
}

impl AxError {
    /// Returns the error description.
    pub fn as_str(&self) -> &'static str {
        use AxError::*;
        match *self {
            BadState => "Bad internal state",
            InvalidData => "Invalid data",
            Unsupported => "Operation not supported",
            UnexpectedEof => "Unexpected end of file",
            WriteZero => "Write zero",
            _ => LinuxError::from(*self).as_str(),
        }
    }
}

impl From<AxError> for LinuxError {
    fn from(e: AxError) -> Self {
        use AxError::*;
        match e {
            AddrInUse => LinuxError::EADDRINUSE,
            AlreadyExists => LinuxError::EEXIST,
            BadAddress | BadState => LinuxError::EFAULT,
            ConnectionRefused => LinuxError::ECONNREFUSED,
            DirectoryNotEmpty => LinuxError::ENOTEMPTY,
            InvalidInput | InvalidData => LinuxError::EINVAL,
            Io => LinuxError::EIO,
            IsADirectory => LinuxError::EISDIR,
            NoMemory => LinuxError::ENOMEM,
            NotADirectory => LinuxError::ENOTDIR,
            NotConnected => LinuxError::ENOTCONN,
            NotFound => LinuxError::ENOENT,
            PermissionDenied => LinuxError::EACCES,
            ResourceBusy => LinuxError::EBUSY,
            StorageFull => LinuxError::ENOSPC,
            Unsupported => LinuxError::ENOSYS,
            UnexpectedEof | WriteZero => LinuxError::EIO,
            WouldBlock => LinuxError::EAGAIN,
        }
    }
}

#[doc(hidden)]
pub mod __priv {
    pub use log::warn;
}

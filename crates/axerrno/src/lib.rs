#![no_std]
#![feature(const_trait_impl)]

mod linux_errno;

pub use linux_errno::LinuxError;

/// The error type used by ArceOS.
#[repr(i32)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxError {
    /// An entity already exists, often a file.
    AlreadyExists,
    /// Try again, often for non-blocking APIs.
    Again,
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
    /// An error returned when an operation could not be completed because a
    /// call to [`write`] returned [`Ok(0)`].
    WriteZero,
}

/// A [`Result`] type with [`AxError`] as the error type.
pub type AxResult<T = ()> = Result<T, AxError>;

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

impl const From<AxError> for LinuxError {
    fn from(e: AxError) -> Self {
        use AxError::*;
        match e {
            AlreadyExists => LinuxError::EEXIST,
            Again => LinuxError::EAGAIN,
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
        }
    }
}

pub mod __priv {
    pub use log::warn;
}

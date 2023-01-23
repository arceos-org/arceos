#![no_std]

mod linux_errno;

pub use linux_errno::LinuxError;

/// The error type used by ArceOS.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxError {
    /// An entity already exists, often a file.
    AlreadyExists,
    /// Bad address.
    BadAddress,
    /// Invalid parameter/argument.
    InvalidParam,
    /// Input/output error.
    Io,
    /// Not enough space/cannot allocate memory.
    NoMemory,
    /// The requested entity is not found.
    NotFound,
    /// The operation lacked the necessary privileges to complete.
    PermissionDenied,
    /// Device or resource is busy.
    ResourceBusy,
    /// This operation is unsupported or unimplemented.
    Unsupported,
}

/// A [`Result`] type with [`RvmError`] as the error type.
pub type AxResult<T = ()> = Result<T, AxError>;

#[macro_export]
macro_rules! ax_err_type {
    ($err: ident) => {{
        use $crate::error::AxError::*;
        warn!("[AxError::{:?}]", $err);
        $err
    }};
    ($err: ident, $msg: expr) => {{
        use $crate::error::AxError::*;
        warn!("[AxError::{:?}] {}", $err, $msg);
        $err
    }};
}

#[macro_export]
macro_rules! ax_err {
    ($err: ident) => {
        Err(rvm_err_type!($err))
    };
    ($err: ident, $msg: expr) => {
        Err(rvm_err_type!($err, $msg))
    };
}

impl From<AxError> for LinuxError {
    fn from(e: AxError) -> Self {
        use AxError::*;
        match e {
            AlreadyExists => LinuxError::EEXIST,
            BadAddress => LinuxError::EFAULT,
            InvalidParam => LinuxError::EINVAL,
            Io => LinuxError::EIO,
            NoMemory => LinuxError::ENOMEM,
            NotFound => LinuxError::ENOENT,
            PermissionDenied => LinuxError::EPERM,
            ResourceBusy => LinuxError::EBUSY,
            Unsupported => LinuxError::ENOSYS,
        }
    }
}

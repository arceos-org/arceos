//! Capture 3. Binary Encoding

/// SBI functions return type
///
/// > SBI functions must return a pair of values in a0 and a1,
/// > with a0 returning an error code.
/// > This is analogous to returning the C structure `SbiRet`.
///
/// Note: if this structure is used in function return on conventional
/// Rust code, it would not require to pin memory representation as
/// extern C. The `repr(C)` is set in case that some users want to use
/// this structure in FFI code.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct SbiRet {
    /// Error number
    pub error: usize,
    /// Result value
    pub value: usize,
}

/// SBI success state return value
pub const RET_SUCCESS: usize = 0;
/// Error for SBI call failed for unknown reasons
pub const RET_ERR_FAILED: usize = -1isize as _;
/// Error for target operation not supported
pub const RET_ERR_NOT_SUPPORTED: usize = -2isize as _;
/// Error for invalid parameter
pub const RET_ERR_INVALID_PARAM: usize = -3isize as _;
/// Error for denied (unused in standard extensions)
pub const RET_ERR_DENIED: usize = -4isize as _;
/// Error for invalid address
pub const RET_ERR_INVALID_ADDRESS: usize = -5isize as _;
/// Error for resource already available
pub const RET_ERR_ALREADY_AVAILABLE: usize = -6isize as _;
/// Error for resource already started
pub const RET_ERR_ALREADY_STARTED: usize = -7isize as _;
/// Error for resource already stopped
pub const RET_ERR_ALREADY_STOPPED: usize = -8isize as _;

impl core::fmt::Debug for SbiRet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.error {
            RET_SUCCESS => self.value.fmt(f),
            RET_ERR_FAILED => write!(f, "<SBI call failed>"),
            RET_ERR_NOT_SUPPORTED => write!(f, "<SBI feature not supported>"),
            RET_ERR_INVALID_PARAM => write!(f, "<SBI invalid parameter>"),
            RET_ERR_DENIED => write!(f, "<SBI denied>"),
            RET_ERR_INVALID_ADDRESS => write!(f, "<SBI invalid address>"),
            RET_ERR_ALREADY_AVAILABLE => write!(f, "<SBI already available>"),
            RET_ERR_ALREADY_STARTED => write!(f, "<SBI already started>"),
            RET_ERR_ALREADY_STOPPED => write!(f, "<SBI already stopped>"),
            unknown => write!(f, "[SBI Unknown error: {unknown:#x}]"),
        }
    }
}

/// RISC-V SBI error in enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Error for SBI call failed for unknown reasons
    Failed,
    /// Error for target operation not supported
    NotSupported,
    /// Error for invalid parameter
    InvalidParam,
    /// Error for denied (unused in standard extensions)
    Denied,
    /// Error for invalid address
    InvalidAddress,
    /// Error for resource already available
    AlreadyAvailable,
    /// Error for resource already started
    AlreadyStarted,
    /// Error for resource already stopped
    AlreadyStopped,
    /// Custom error code
    Custom(isize),
}

impl SbiRet {
    /// Returns success SBI state with given `value`.
    #[inline]
    pub const fn success(value: usize) -> Self {
        Self {
            error: RET_SUCCESS,
            value,
        }
    }

    /// The SBI call request failed for unknown reasons.
    #[inline]
    pub const fn failed() -> Self {
        Self {
            error: RET_ERR_FAILED,
            value: 0,
        }
    }

    /// SBI call failed due to not supported by target ISA,
    /// operation type not supported,
    /// or target operation type not implemented on purpose.
    #[inline]
    pub const fn not_supported() -> Self {
        Self {
            error: RET_ERR_NOT_SUPPORTED,
            value: 0,
        }
    }

    /// SBI call failed due to invalid hart mask parameter,
    /// invalid target hart id,
    /// invalid operation type,
    /// or invalid resource index.
    #[inline]
    pub const fn invalid_param() -> Self {
        Self {
            error: RET_ERR_INVALID_PARAM,
            value: 0,
        }
    }

    /// SBI call failed for invalid mask start address,
    /// not a valid physical address parameter,
    /// or the target address is prohibited by PMP to run in supervisor mode.
    #[inline]
    pub const fn invalid_address() -> Self {
        Self {
            error: RET_ERR_INVALID_ADDRESS,
            value: 0,
        }
    }

    /// SBI call failed for the target resource is already available,
    /// e.g. the target hart is already started when caller still request it to start.
    #[inline]
    pub const fn already_available() -> Self {
        Self {
            error: RET_ERR_ALREADY_AVAILABLE,
            value: 0,
        }
    }

    /// SBI call failed for the target resource is already started,
    /// e.g. target performance counter is started.
    #[inline]
    pub const fn already_started() -> Self {
        Self {
            error: RET_ERR_ALREADY_STARTED,
            value: 0,
        }
    }

    /// SBI call failed for the target resource is already stopped,
    /// e.g. target performance counter is stopped.
    #[inline]
    pub const fn already_stopped() -> Self {
        Self {
            error: RET_ERR_ALREADY_STOPPED,
            value: 0,
        }
    }
}

impl SbiRet {
    /// Converts to a [`Result`] of value and error.
    #[inline]
    pub const fn into_result(self) -> Result<usize, Error> {
        match self.error {
            RET_SUCCESS => Ok(self.value),
            RET_ERR_FAILED => Err(Error::Failed),
            RET_ERR_NOT_SUPPORTED => Err(Error::NotSupported),
            RET_ERR_INVALID_PARAM => Err(Error::InvalidParam),
            RET_ERR_DENIED => Err(Error::Denied),
            RET_ERR_INVALID_ADDRESS => Err(Error::InvalidAddress),
            RET_ERR_ALREADY_AVAILABLE => Err(Error::AlreadyAvailable),
            RET_ERR_ALREADY_STARTED => Err(Error::AlreadyStarted),
            RET_ERR_ALREADY_STOPPED => Err(Error::AlreadyStopped),
            unknown => Err(Error::Custom(unknown as _)),
        }
    }

    /// Returns `true` if current SBI return succeeded.
    #[must_use = "if you intended to assert that this is ok, consider `.unwrap()` instead"]
    #[inline]
    pub const fn is_ok(&self) -> bool {
        matches!(self.error, RET_SUCCESS)
    }

    /// Returns `true` if current SBI return is an error.
    #[must_use = "if you intended to assert that this is err, consider `.unwrap_err()` instead"]
    #[inline]
    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Converts from `SbiRet` to [`Option<usize>`].
    // fixme: should be pub const fn once this function in Result is stablized in constant
    #[inline]
    pub fn ok(self) -> Option<usize> {
        self.into_result().ok()
    }

    /// Converts from `SbiRet` to [`Option<Error>`].
    // fixme: should be pub const fn once this function in Result is stablized in constant
    #[inline]
    pub fn err(self) -> Option<Error> {
        self.into_result().err()
    }

    /// Maps a `SbiRet` to `Result<U, E>` by applying a function to a
    /// contained success value, leaving an error value untouched.
    #[inline]
    pub fn map<U, F: FnOnce(usize) -> U>(self, op: F) -> Result<U, Error> {
        self.into_result().map(op)
    }

    /// Returns the provided default (if error), or
    /// applies a function to the contained value (if success),
    #[inline]
    pub fn map_or<U, F: FnOnce(usize) -> U>(self, default: U, f: F) -> U {
        self.into_result().map_or(default, f)
    }

    /// Maps a `SbiRet` to `usize` value by applying fallback function `default` to
    /// a contained error, or function `f` to a contained success value.
    #[inline]
    pub fn map_or_else<U, D: FnOnce(Error) -> U, F: FnOnce(usize) -> U>(
        self,
        default: D,
        f: F,
    ) -> U {
        self.into_result().map_or_else(default, f)
    }

    /// Maps a `SbiRet` to `Result<T, F>` by applying a function to a
    /// contained error as [`Error`] struct, leaving success value untouched.
    #[inline]
    pub fn map_err<F, O: FnOnce(Error) -> F>(self, op: O) -> Result<usize, F> {
        self.into_result().map_err(op)
    }

    /// Returns the contained success value, consuming the `self` value.
    ///
    /// # Panics
    ///
    /// Panics if self is an SBI error with a panic message including the
    /// passed message, and the content of the SBI state.
    #[inline]
    pub fn expect(self, msg: &str) -> usize {
        self.into_result().expect(msg)
    }

    /// Returns the contained success value, consuming the `self` value.
    ///
    /// # Panics
    ///
    /// Panics if self is an SBI error, with a panic message provided by the
    /// SBI error converted into [`Error`] struct.
    #[inline]
    pub fn unwrap(self) -> usize {
        self.into_result().unwrap()
    }

    /// Returns the contained error as [`Error`] struct, consuming the `self` value.
    ///
    /// # Panics
    ///
    /// Panics if the self is SBI success value, with a panic message including the
    /// passed message, and the content of the success value.
    #[inline]
    pub fn expect_err(self, msg: &str) -> Error {
        self.into_result().expect_err(msg)
    }

    /// Returns the contained error as [`Error`] struct, consuming the `self` value.
    ///
    /// # Panics
    ///
    /// Panics if the self is SBI success value, with a custom panic message provided
    /// by the success value.
    #[inline]
    pub fn unwrap_err(self) -> Error {
        self.into_result().unwrap_err()
    }

    /// Returns `res` if self is success value, otherwise otherwise returns the contained error
    /// of `self` as [`Error`] struct.
    // fixme: should be pub const fn once this function in Result is stablized in constant
    #[inline]
    pub fn and(self, res: Result<usize, Error>) -> Result<usize, Error> {
        self.into_result().and(res)
    }

    /// Calls `op` if self is success value, otherwise returns the contained error
    /// as [`Error`] struct.
    #[inline]
    pub fn and_then<U, F: FnOnce(usize) -> Result<U, Error>>(self, op: F) -> Result<U, Error> {
        self.into_result().and_then(op)
    }

    /// Returns `res` if self is SBI error, otherwise returns the success value of `self`.
    // fixme: should be pub const fn once this function in Result is stablized in constant
    #[inline]
    pub fn or<F>(self, res: Result<usize, F>) -> Result<usize, F> {
        self.into_result().or(res)
    }

    /// Calls `op` if self is SBI error, otherwise returns the success value of `self`.
    #[inline]
    pub fn or_else<F, O: FnOnce(Error) -> Result<usize, F>>(self, op: O) -> Result<usize, F> {
        self.into_result().or_else(op)
    }

    /// Returns the contained success value or a provided default.
    // fixme: should be pub const fn once this function in Result is stablized in constant
    #[inline]
    pub fn unwrap_or(self, default: usize) -> usize {
        self.into_result().unwrap_or(default)
    }

    /// Returns the contained success value or computes it from a closure.
    #[inline]
    pub fn unwrap_or_else<F: FnOnce(Error) -> usize>(self, op: F) -> usize {
        self.into_result().unwrap_or_else(op)
    }
}

/// JBD Error type.
#[derive(Debug)]
pub enum JBDError {
    /// The given journal size is invalid.
    InvalidJournalSize,
    /// The superblock on the disk is corrupted.
    InvalidSuperblock,
    /// The journal is running out of space.
    NotEnoughSpace,
    /// The journal has aborted.
    JournalAborted,
    /// The handle has aborted.
    HandleAborted,
    /// The transaction is not in the `Running` state.
    TransactionNotRunning,
    /// IO error.
    IOError,
    /// An unexpected error.
    Unknown,
}

/// Type alias for `Result<_ JBDError>`.
pub type JBDResult<T = ()> = Result<T, JBDError>;

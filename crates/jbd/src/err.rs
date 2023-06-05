#[derive(Debug)]
pub enum JBDError {
    // Buffer
    InsufficientCache,
    CacheNotFound,
    // Journal
    InvalidJournalSize,
    InvalidSuperblock,
    NotEnoughSpace,
    JournalAborted,
    // Handle
    HandleAborted,
    TransactionNotRunning,
    // Misc
    IOError,
    Unknown,
}

pub type JBDResult<T = ()> = Result<T, JBDError>;

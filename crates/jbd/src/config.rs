/// The default maximum commit age, in seconds.
// pub const JBD_DEFAULT_MAX_COMMIT_AGE: usize = 5;
/// Minumum number of blocks to reserve for the log.
pub const JFS_MIN_JOURNAL_BLOCKS: u32 = 1024;
/// The magic number.
pub const JFS_MAGIC_NUMBER: u32 = 0xc03b3998;

// pub const JOURNAL_REVOKE_DEFAULT_HASH: usize = 256;

// #[cfg(feature = "debug")]
// pub const JBD_EXPENSIVE_CHECKING: bool = true;
// #[cfg(not(feature = "debug"))]
// pub const JBD_EXPENSIVE_CHECKING: bool = false;

pub const MIN_LOG_RESERVED_BLOCKS: u32 = 32;

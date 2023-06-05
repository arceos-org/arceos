/// Number of blocks for the journal
#[cfg(feature = "journal")]
pub const JOURNAL_SIZE: u32 = 1024;
#[cfg(not(feature = "journal"))]
pub const JOURNAL_SIZE: u32 = 0;

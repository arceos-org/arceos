/// Number of blocks for the journal
#[cfg(feature = "journal")]
pub const JOURNAL_SIZE: u32 = 8192;
#[cfg(not(feature = "journal"))]
pub const JOURNAL_SIZE: u32 = 0;

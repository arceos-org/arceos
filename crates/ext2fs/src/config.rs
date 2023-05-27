#![allow(unused)]
/// Block size used by ext2 file system
pub const BLOCK_SIZE: usize = 2048;
pub(crate) const LOG_BLOCK_SIZE: usize = 1;
pub(crate) const LOG_FRAG_SIZE: usize = LOG_BLOCK_SIZE;

pub(crate) const INODES_PER_GRP: usize = 8 * BLOCK_SIZE;
/// Blocks per group
pub const BLOCKS_PER_GRP: usize = 8 * BLOCK_SIZE;

pub(crate) const FIRST_DATA_BLOCK: usize = if BLOCK_SIZE > 1024 { 0 } else { 1 };
pub(crate) const SUPER_BLOCK_OFFSET: usize = if FIRST_DATA_BLOCK == 0 { 1024 } else { 0 };

pub(crate) const FAKE_CREATE_TIME: usize = 50 * 365 * 24 * 3600;
pub(crate) const CHECK_INTERVAL: usize = 3 * 30 * 24 * 3600;

pub(crate) const EXT2_GOOD_OLD_FIRST_INO: usize = 11;
pub(crate) const EXT2_GOOD_OLD_INODE_SIZE: usize = 128;
pub(crate) const INODE_TABLE_BLOCK_NUM: usize =
    (INODES_PER_GRP * EXT2_GOOD_OLD_INODE_SIZE) / BLOCK_SIZE;
pub(crate) const RESERVED_BLOCKS_PER_GRP: usize = INODE_TABLE_BLOCK_NUM + 2;

pub(crate) const FAKE_UUID: u128 = 114514;
pub(crate) const FAKE_JOURNAL_UUID: u128 = 996;

pub(crate) const EXT2_ROOT_INO: usize = 2;
pub(crate) const MAX_PATH_NAME: usize = 1024;

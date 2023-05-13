#![allow(unused)]
pub const BLOCK_SIZE: usize = 2048;
pub const LOG_BLOCK_SIZE: usize = 1;
pub const LOG_FRAG_SIZE: usize = LOG_BLOCK_SIZE;

pub const INODES_PER_GRP: usize = 8 * BLOCK_SIZE;
pub const BLOCKS_PER_GRP: usize = 8 * BLOCK_SIZE;

pub const FIRST_DATA_BLOCK: usize = if BLOCK_SIZE > 1024 { 0 } else { 1 };
pub const SUPER_BLOCK_OFFSET: usize = if FIRST_DATA_BLOCK == 0 { 1024 } else { 0 };

pub const FAKE_CREATE_TIME: usize = 50 * 365 * 24 * 3600;
pub const CHECK_INTERVAL: usize = 3 * 30 * 24 * 3600;

pub const EXT2_GOOD_OLD_FIRST_INO: usize = 11;
pub const EXT2_GOOD_OLD_INODE_SIZE: usize = 128;
pub const INODE_TABLE_BLOCK_NUM: usize = (INODES_PER_GRP * EXT2_GOOD_OLD_INODE_SIZE) / BLOCK_SIZE;
pub const RESERVED_BLOCKS_PER_GRP: usize = INODE_TABLE_BLOCK_NUM + 2;

pub const FAKE_UUID: u128 = 114514;
pub const FAKE_JOURNAL_UUID: u128 = 996;

pub const EXT2_ROOT_INO: usize = 2;
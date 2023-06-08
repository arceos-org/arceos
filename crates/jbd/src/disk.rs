//! On-disk structures for the journal.

#[cfg(feature = "debug")]
extern crate alloc;
#[cfg(feature = "debug")]
use alloc::string::{String, ToString};
use bitflags::bitflags;
use cfg_if::cfg_if;

use crate::err::{JBDError, JBDResult};

/// Descriptor block types.
#[derive(Clone, Copy, Debug)]
pub(crate) enum BlockType {
    DescriptorBlock = 1,
    CommitBlock = 2,
    SuperblockV1 = 3,
    SuperblockV2 = 4,
    RevokeBlock = 5,
}

impl BlockType {
    pub(crate) fn from_u32_be(block_type: u32) -> JBDResult<Self> {
        let block_type = u32::from_be(block_type);
        match block_type {
            1 => Ok(BlockType::DescriptorBlock),
            2 => Ok(BlockType::CommitBlock),
            3 => Ok(BlockType::SuperblockV1),
            4 => Ok(BlockType::SuperblockV2),
            5 => Ok(BlockType::RevokeBlock),
            _ => Err(JBDError::InvalidSuperblock),
        }
    }

    pub(crate) fn to_u32_be(self) -> u32 {
        let val = match self {
            BlockType::DescriptorBlock => 1,
            BlockType::CommitBlock => 2,
            BlockType::SuperblockV1 => 3,
            BlockType::SuperblockV2 => 4,
            BlockType::RevokeBlock => 5,
        } as u32;
        val.to_be()
    }
}

bitflags! {
    #[derive(Default)]
    #[repr(C)]
    pub(crate) struct TagFlag: u32 {
        const ESCAPE = 1;
        const SAME_UUID = 1 << 1;
        const DELETED = 1 << 2;
        const LAST_TAG = 1 << 3;
    }
}

/// Standard header for all descriptor blocks.
#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) struct Header {
    pub(crate) magic: u32,
    pub(crate) block_type: u32,
    pub(crate) sequence: u32,
}

/// Used to describe a single buffer in the journal.
#[repr(C)]
pub(crate) struct BlockTag {
    /// The on-disk block number
    pub(crate) block_nr: u32,
    pub(crate) flag: u32,
}

/// The revoke descriptor: used on disk to describe a series of blocks to be revoked from the log
#[repr(C)]
pub(crate) struct RevokeBlockHeader {
    pub(crate) header: Header,
    pub(crate) count: u32,
}

/// The journal superblock. All fields are in big-endian byte order.
#[repr(C)]
pub(crate) struct Superblock {
    pub(crate) header: Header,

    /* Static information describing the journal */
    /// Journal device blocksize
    pub(crate) block_size: u32,
    /// Yotal blocks in journal file
    pub(crate) maxlen: u32,
    /// First block of log information
    pub(crate) first: u32,

    /* Dynamic information describing the current state of the log */
    /// First commit ID expected in log
    pub(crate) sequence: u32,
    /// Block_nr of start of log
    pub(crate) start: u32,

    /* Error value, as set by journal_abort(). */
    pub(crate) errno: i32,

    /* Remaining fields are only valid in a version-2 superblock */
    /// Compatible feature set
    pub(crate) feature_compat: u32,
    /// Incompatible feature set
    pub(crate) feature_incompat: u32,
    /// Readonly-compatible feature set
    pub(crate) feature_ro_compat: u32,
    /// UUID of journal superblock
    pub(crate) uuid: [u8; 16],
    /// Number of filesystems sharing log
    pub(crate) nr_users: u32,
    /// Blocknr of dynamic superblock copy
    pub(crate) dyn_super: u32,
    /// Limit of journal blocks per trans
    pub(crate) max_transaction: u32,
    /// Limit of data blocks per trans
    pub(crate) max_trans_data: u32,
    pub(crate) padding: [u32; 44],
    /// Ids of all fs'es sharing the log
    pub(crate) users: [u8; 16 * 48],
}

cfg_if! {
if #[cfg(feature = "debug")] {

pub(crate) trait Display {
    fn display(&self, ident: usize) -> String;
}

fn get_ident(ident: usize) -> String {
    let mut str = String::new();
    str += "\n";
    for _ in 0..ident {
        str += "  ";
    }
    str
}

impl Display for BlockType {
    fn display(&self, _ident: usize) -> String {
        match self {
            BlockType::DescriptorBlock => "DescriptorBlock".to_string(),
            BlockType::CommitBlock => "CommitBlock".to_string(),
            BlockType::SuperblockV1 => "SuperblockV1".to_string(),
            BlockType::SuperblockV2 => "SuperblockV2".to_string(),
            BlockType::RevokeBlock => "Revokeblock".to_string(),
        }
    }
}

impl Display for TagFlag {
    fn display(&self, ident: usize) -> String {
        get_ident(ident)
            + &format_args!(
                "ESCAPE: {}, SAME_UUID: {}, DELETED: {}, LAST_TAG: {}",
                self.contains(TagFlag::ESCAPE),
                self.contains(TagFlag::SAME_UUID),
                self.contains(TagFlag::DELETED),
                self.contains(TagFlag::LAST_TAG),
            )
            .to_string()
    }
}

impl Display for Header {
    fn display(&self, ident: usize) -> String {
        let ident_str = get_ident(ident);
        let block_type = BlockType::from_u32_be(self.block_type).unwrap();
        format_args!(
            "{}magic: {:x}{}block_type: {}{}sequence: {}",
            &ident_str,
            u32::from_be(self.magic),
            &ident_str,
            block_type.display(ident + 1),
            &ident_str,
            u32::from_be(self.sequence),
        )
        .to_string()
    }
}

impl Display for BlockTag {
    fn display(&self, ident: usize) -> String {
        let ident_str = get_ident(ident);
        format_args!(
            "{}block_nr: {}{}flag: {}",
            &ident_str,
            u32::from_be(self.block_nr),
            &ident_str,
            TagFlag::from_bits(u32::from_be(self.flag)).unwrap().display(ident + 1),
        )
        .to_string()
    }
}

impl Display for Superblock {
    fn display(&self, ident: usize) -> String {
        let ident_str = get_ident(ident);
        format_args!(
            "{}header: {}{}block_size: {}{}maxlen: {}{}first: {}{}sequence: {}{}start: {}{}errno: {}",
            &ident_str,
            self.header.display(ident + 1),
            &ident_str,
            u32::from_be(self.block_size),
            &ident_str,
            u32::from_be(self.maxlen),
            &ident_str,
            u32::from_be(self.first),
            &ident_str,
            u32::from_be(self.sequence),
            &ident_str,
            u32::from_be(self.start),
            &ident_str,
            i32::from_be(self.errno),
        )
        .to_string()
    }
}

}
}

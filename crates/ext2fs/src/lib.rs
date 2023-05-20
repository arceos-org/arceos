#![no_std]
#![feature(allocator_api)]
#![feature(new_uninit)]
extern crate alloc;
pub mod layout;
mod config;
mod block_dev;
mod bitmap;
mod efs;
mod vfs;
pub mod timer;
pub mod ext2err;
mod block_cache_manager;
mod inode_manager;
mod mutex;

pub use block_dev::BlockDevice;
pub use efs::Ext2FileSystem;
pub use vfs::Inode;
use vfs::InodeCache;
pub use timer::{TimeProvider, ZeroTimeProvider};
pub use config::{BLOCK_SIZE, BLOCKS_PER_GRP};
pub use layout::{EXT2_S_IFREG, EXT2_S_IFDIR};
pub use ext2err::{Ext2Error, Ext2Result};
use bitmap::Bitmap;
use layout::{SuperBlock, DiskInode, BlockGroupDesc};

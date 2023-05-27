//! An ext2 file system implementation

#![no_std]
#![feature(allocator_api)]
#![feature(new_uninit)]
extern crate alloc;
mod bitmap;
mod block_cache_manager;
mod block_dev;
mod config;
mod efs;
/// Ext2 error
pub mod ext2err;
mod inode_manager;
/// Ext2 layout
pub mod layout;
mod mutex;
/// Timer
pub mod timer;
mod vfs;

use bitmap::Bitmap;
pub use block_dev::BlockDevice;
pub use config::{BLOCKS_PER_GRP, BLOCK_SIZE};
pub use efs::Ext2FileSystem;
pub use ext2err::{Ext2Error, Ext2Result};
use layout::{BlockGroupDesc, DiskInode, SuperBlock};
pub use layout::{EXT2_S_IFDIR, EXT2_S_IFREG};
pub use timer::{TimeProvider, ZeroTimeProvider};
pub use vfs::Inode;
use vfs::InodeCache;

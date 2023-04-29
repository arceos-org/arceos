//! [ArceOS](https://github.com/rcore-os/arceos) filesystem module.
//!
//! It provides unified filesystem operations for various filesystems.
//!
//! # Cargo Features
//!
//! - `use-ramdisk`: Use [`driver_block::ramdisk::RamDisk`] as the block device.
//!    This feature is **enabled** by default.
//! - `use-virtio-blk`: Use [`axdriver::VirtIoBlockDev`] as the block device.
//!    This feature is **disabled** by default, but it will override `use-ramdisk`
//!    if both are enabled.
//! - `fatfs`: Use [FAT] as the main filesystem and mount it on `/`. This feature
//!    is **enabled** by default.
//! - `devfs`: Mount [`axfs_devfs::DeviceFileSystem`] on `/dev`. This feature is
//!    **enabled** by default.
//! - `ramfs`: Mount [`axfs_ramfs::RamFileSystem`] on `/tmp`. This feature is
//!    **enabled** by default.
//! - `myfs`: Allow users to define their custom filesystems to override the
//!   default. In this case, [`MyFileSystemIf`] is required to be implemented
//!   to create and initialize other filesystems. This feature is **disabled** by
//!    by default, but it will override
//!   other filesystem selection features if both are enabled.
//!
//! [FAT]: https://en.wikipedia.org/wiki/File_Allocation_Table
//! [`MyFileSystemIf`]: fops::MyFileSystemIf

#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_auto_cfg)]

#[macro_use]
extern crate log;
extern crate alloc;

mod dev;
mod fs;
mod root;

pub mod api;
pub mod fops;

cfg_if::cfg_if! {
    if #[cfg(feature = "use-virtio-blk")] {
        use axdriver::VirtIoBlockDev as BlockDevice;
    } else if #[cfg(feature = "use-ramdisk")] {
        use driver_block::ramdisk::RamDisk as BlockDevice;
    }
}

use driver_block::BaseDriverOps;

/// Initializes filesystems by the given block device.
///
/// If the feature `use-virtio-blk` is enabled, `BlockDevice` is an alias of
/// [`axdriver::VirtIoBlockDev`].
///
/// Otherwise, if the feature `use-ramdisk` is enabled, `BlockDevice` is an
/// alias of [`driver_block::ramdisk::RamDisk`].
pub fn init_filesystems(blk_dev: BlockDevice) {
    info!("Initialize filesystems...");
    info!("  use block device: {:?}", blk_dev.device_name());

    let disk = self::dev::Disk::new(blk_dev);
    self::root::init_rootfs(disk);
}

//! [ArceOS](https://github.com/rcore-os/arceos) filesystem module.
//!
//! It provides unified filesystem operations for various filesystems.
//!
//! # Cargo Features
//!
//! - `fatfs`: Use [FAT] as the main filesystem and mount it on `/`. This feature
//!    is **enabled** by default.
//! - `devfs`: Mount [`axfs_devfs::DeviceFileSystem`] on `/dev`. This feature is
//!    **enabled** by default.
//! - `ramfs`: Mount [`axfs_ramfs::RamFileSystem`] on `/tmp`. This feature is
//!    **enabled** by default.
//! - `myfs`: Allow users to define their custom filesystems to override the
//!    default. In this case, [`MyFileSystemIf`] is required to be implemented
//!    to create and initialize other filesystems. This feature is **disabled** by
//!    by default, but it will override other filesystem selection features if
//!    both are enabled.
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

use axdriver::{prelude::*, AxDeviceContainer};

/// Initializes filesystems by block devices.
pub fn init_filesystems(
    #[allow(unused, unused_mut)] mut blk_devs: AxDeviceContainer<AxBlockDevice>,
) {
    #[cfg(not(feature = "user"))]
    {
        info!("Initialize filesystems...");

        let dev = blk_devs.take_one().expect("No block device found!");
        info!("  use block device 0: {:?}", dev.device_name());
        self::root::init_rootfs(self::dev::Disk::new(dev));
    }
}

#[cfg(feature = "user")]
pub use user::user_init;
#[cfg(feature = "user")]
mod user {
    use axerrno::{AxError, AxResult};
    use axio::{Read, Seek, Write};
    use driver_block::{DevError, DevResult};
    use libax::io::File;

    pub fn user_init() {
        let dev = AxBlockDeviceMock::new().unwrap();
        super::root::init_rootfs(super::dev::Disk::new(dev));
    }

    pub struct AxBlockDeviceMock {
        file: File,
        block_size: usize,
        num_blocks: u64,
    }

    impl AxBlockDeviceMock {
        fn new() -> AxResult<Self> {
            let mut block_size: usize = 0;
            assert_eq!(
                File::open("dev:/disk/block_size")
                    .map_err(|_| AxError::Unsupported)?
                    .read_data(&mut block_size),
                Ok(core::mem::size_of_val(&block_size))
            );

            let mut num_blocks: u64 = 0;
            assert_eq!(
                File::open("dev:/disk/num_blocks")
                    .map_err(|_| AxError::Unsupported)?
                    .read_data(&mut num_blocks),
                Ok(core::mem::size_of_val(&num_blocks))
            );

            Ok(AxBlockDeviceMock {
                file: File::open("dev:/disk").map_err(|_| AxError::Unsupported)?,
                block_size,
                num_blocks,
            })
        }
        pub fn block_size(&self) -> usize {
            self.block_size
        }

        pub fn num_blocks(&self) -> u64 {
            self.num_blocks
        }

        pub fn read_block(&mut self, block_id: u64, buf: &mut [u8]) -> DevResult {
            self.file
                .seek(axio::SeekFrom::Start(block_id * self.block_size as u64))
                .map_err(map_err)?;
            self.file.read_exact(buf).map_err(map_err)
        }
        pub fn write_block(&mut self, block_id: u64, buf: &[u8]) -> DevResult {
            self.file
                .seek(axio::SeekFrom::Start(block_id * self.block_size as u64))
                .map_err(map_err)?;
            self.file.write_all(buf).map_err(map_err)
        }
    }

    fn map_err(e: AxError) -> DevError {
        match e {
            AxError::Again => DevError::Again,
            AxError::AlreadyExists => DevError::AlreadyExists,
            AxError::BadState => DevError::BadState,
            AxError::InvalidInput => DevError::InvalidParam,
            AxError::Io => DevError::Io,
            AxError::NoMemory => DevError::NoMemory,
            AxError::ResourceBusy => DevError::ResourceBusy,
            AxError::Unsupported => DevError::Unsupported,
            _ => DevError::Io,
        }
    }
}

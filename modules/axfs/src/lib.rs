#![no_std]

#[macro_use]
extern crate log;
extern crate alloc;

mod dev;
mod fs;
mod root;

pub mod api;
pub mod fops;

use driver_common::BaseDriverOps;

type BlockDevice = axdriver::VirtIoBlockDev;

pub fn init_filesystems(blk_dev: BlockDevice) {
    info!("Initialize filesystems...");
    info!("  use block device: {:?}", blk_dev.device_name());

    let disk = self::dev::Disk::new(blk_dev);
    self::root::init_rootfs(disk);
}

#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate log;
extern crate alloc;

mod dev;
mod fs;
mod root;

pub mod api;
pub mod fops;

use driver_common::BaseDriverOps;

cfg_if::cfg_if! {
    if #[cfg(feature = "use-virtio-blk")] {
        type BlockDevice = axdriver::VirtIoBlockDev;
    } else if #[cfg(feature = "use-ramdisk")] {
        type BlockDevice = driver_block::ramdisk::RamDisk;
    }
}

pub fn init_filesystems(blk_dev: BlockDevice) {
    info!("Initialize filesystems...");
    info!("  use block device: {:?}", blk_dev.device_name());

    let disk = self::dev::Disk::new(blk_dev);
    self::root::init_rootfs(disk);
}

#![cfg(not(feature = "myfs"))]

mod test_common;

use axdriver::AxDeviceContainer;
use driver_block::ramdisk::RamDisk;

const IMG_PATH: &str = "resources/fat16.img";

fn make_disk() -> std::io::Result<RamDisk> {
    let path = std::env::current_dir()?.join(IMG_PATH);
    println!("Loading disk image from {:?} ...", path);
    let data = std::fs::read(path)?;
    println!("size = {} bytes", data.len());
    Ok(RamDisk::from(&data))
}

#[test]
fn test_fatfs() {
    println!("Testing fatfs with ramdisk ...");

    let disk = make_disk().expect("failed to load disk image");
    axtask::init_scheduler(); // call this to use `axsync::Mutex`.
    axfs::init_filesystems(AxDeviceContainer::from_one(disk));

    test_common::test_all();
}

#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

#[macro_use]
extern crate alloc;

use core::arch::global_asm;
global_asm!(include_str!("link_fs.S"));

use axdriver::AxDeviceContainer;
use axfs::api as fs;
use axio as io;
use driver_block::ramdisk::RamDisk;

use io::Result;

fn test_read_dir() -> Result<()> {
    let dir = "/././//./";
    println!("list directory {:?}:", dir);
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        println!("   {}", entry.file_name());
    }
    println!("test_read_dir() OK!");
    Ok(())
}


#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("Hello, world!");

    extern "C" {
        fn fs_();
        fn fs_start();
        fn fs_end();
    }
    println!("fs addr {:#x?}", fs_ as usize);
    println!("fs_start {:#x?}", fs_start as usize);
    println!("fs_start {:#x?}", fs_end as usize);

    let fs_data: &[u8] = unsafe {
        core::slice::from_raw_parts(fs_start as *mut u8, fs_end as usize - fs_start as usize)
    };

    let ramdisk = RamDisk::from(&fs_data);
    axfs::init_filesystems(AxDeviceContainer::from_one(ramdisk));

    test_read_dir().expect("test_read_dir() failed");
}

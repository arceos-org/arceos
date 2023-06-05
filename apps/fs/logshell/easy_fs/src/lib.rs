//!An easy file system isolated from the kernel
#![no_std]
#![feature(trait_alias)]

extern crate alloc;
mod bitmap;
mod block_cache;
mod block_dev;
mod config;
mod efs;
#[cfg(feature = "journal")]
mod journal;
mod layout;
mod vfs;
/// Use a block size of 512 bytes
pub const BLOCK_SZ: usize = 512;
use alloc::{rc::Rc, sync::Arc};
use axfs_vfs::VfsOps;
use bitmap::Bitmap;
use block_cache::{block_cache_sync_all, get_block_cache};

#[cfg(not(feature = "journal"))]
pub use block_dev::BlockDevice;
pub use efs::EasyFileSystem;
use efs::EasyFileSystemWrapper;
#[cfg(feature = "journal")]
use jbd::sal::BlockDevice;
pub use layout::DiskInodeType;
use layout::*;
pub use spin::Mutex;
pub use vfs::Inode;

use axfs::fops::{Disk, MyFileSystemIf};
struct MyFileSystemIfImpl;

struct DiskWrapper(Mutex<Disk>);

impl BlockDevice for DiskWrapper {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let mut disk = self.0.lock();
        disk.set_position(block_id as u64);
        disk.read_one(buf).unwrap();
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let mut disk = self.0.lock();
        disk.set_position(block_id as u64);
        disk.write_one(buf).unwrap();
    }

    #[cfg(feature = "journal")]
    fn block_size(&self) -> usize {
        BLOCK_SZ
    }
}

static mut DISK: Option<Rc<DiskWrapper>> = None;
pub static mut MAIN_FS: Option<Arc<EasyFileSystemWrapper>> = None;

#[crate_interface::impl_interface]
impl MyFileSystemIf for MyFileSystemIfImpl {
    fn new_myfs(disk: Disk) -> Arc<dyn VfsOps> {
        let disk = Rc::new(DiskWrapper(Mutex::new(disk)));
        let fs = Arc::new(EasyFileSystemWrapper::new(EasyFileSystem::create(
            disk.clone(),
            16 * 2048,
            16,
        )));
        unsafe {
            DISK = Some(disk);
            MAIN_FS = Some(fs.clone());
        }
        fs
    }
}

pub fn crash() {
    block_cache_sync_all();

    unsafe {
        MAIN_FS
            .as_ref()
            .unwrap()
            .as_ref()
            .inner()
            .lock()
            .load(DISK.as_ref().unwrap().clone());
    }
}

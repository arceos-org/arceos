extern crate alloc;

use alloc::sync::Arc;
use axfs::fops::{Disk, MyFileSystemIf};
use axfs_ramfs::RamFileSystem;
use axfs_vfs::VfsOps;

struct MyFileSystemIfImpl;

#[crate_interface::impl_interface]
impl MyFileSystemIf for MyFileSystemIfImpl {
    fn new_myfs(_disk: Disk) -> Arc<dyn VfsOps> {
        Arc::new(RamFileSystem::new())
    }
}

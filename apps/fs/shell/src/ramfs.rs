extern crate alloc;

use alloc::sync::Arc;
use axfs_ramfs::RamFileSystem;
use axfs_vfs::VfsOps;
use std::os::arceos::axfs::fops::{Disk, MyFileSystemIf};

struct MyFileSystemIfImpl;

#[crate_interface::impl_interface]
impl MyFileSystemIf for MyFileSystemIfImpl {
    fn new_myfs(_disk: Disk) -> Arc<dyn VfsOps> {
        Arc::new(RamFileSystem::new())
    }
}

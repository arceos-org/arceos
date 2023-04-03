#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod dir;
mod null;
mod zero;

#[cfg(test)]
mod tests;

pub use self::dir::DirNode;
pub use self::null::NullDev;
pub use self::zero::ZeroDev;

use alloc::sync::Arc;
use axfs_vfs::{VfsNodeRef, VfsOps};

#[derive(Default)]
pub struct DeviceFileSystem {
    root: Arc<DirNode>,
}

impl DeviceFileSystem {
    pub fn new() -> Self {
        Self {
            root: Arc::new(DirNode::new()),
        }
    }

    pub fn add(&mut self, name: &'static str, node: VfsNodeRef) {
        let root = Arc::get_mut(&mut self.root).unwrap();
        root.add(name, node);
    }
}

impl VfsOps for DeviceFileSystem {
    fn root_dir(&self) -> VfsNodeRef {
        self.root.clone()
    }
}

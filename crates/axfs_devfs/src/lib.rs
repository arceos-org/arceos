//! Device filesystem.

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
use axfs_vfs::{VfsNodeRef, VfsOps, VfsResult};
use spin::once::Once;

pub struct DeviceFileSystem {
    parent: Once<VfsNodeRef>,
    root: Arc<DirNode>,
}

impl DeviceFileSystem {
    pub fn new() -> Self {
        Self {
            parent: Once::new(),
            root: DirNode::new(None),
        }
    }

    pub fn mkdir(&self, name: &'static str) -> Arc<DirNode> {
        self.root.mkdir(name)
    }

    pub fn add(&self, name: &'static str, node: VfsNodeRef) {
        self.root.add(name, node);
    }
}

impl VfsOps for DeviceFileSystem {
    fn mount(&self, _path: &str, mount_point: VfsNodeRef) -> VfsResult {
        if let Some(parent) = mount_point.parent() {
            self.root.set_parent(Some(self.parent.call_once(|| parent)));
        } else {
            self.root.set_parent(None);
        }
        Ok(())
    }

    fn root_dir(&self) -> VfsNodeRef {
        self.root.clone()
    }
}

impl Default for DeviceFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

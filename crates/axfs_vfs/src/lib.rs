#![no_std]

extern crate alloc;

mod macros;
mod structs;

use alloc::sync::Arc;
use axerrno::{ax_err, AxError, AxResult};

pub use self::structs::{FileSystemInfo, VfsDirEntry, VfsNodeAttr, VfsNodePerm, VfsNodeType};

pub type VfsNodeRef = Arc<dyn VfsNodeOps>;

pub type VfsError = AxError;
pub type VfsResult<T = ()> = AxResult<T>;

/// Filesystem operations.
pub trait VfsOps: Send + Sync {
    fn mount(&self, _path: &str) -> VfsResult {
        Ok(())
    }

    fn umount(&self) -> VfsResult {
        Ok(())
    }

    fn format(&self) -> VfsResult {
        ax_err!(Unsupported)
    }

    fn statfs(&self) -> VfsResult<FileSystemInfo> {
        ax_err!(Unsupported)
    }

    fn root_dir(&self) -> VfsNodeRef;
}

/// File (inode) operations.
pub trait VfsNodeOps: Send + Sync {
    fn open(&self) -> VfsResult {
        Ok(())
    }

    fn release(&self) -> VfsResult {
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<VfsNodeAttr>;

    // file operations:

    fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> VfsResult<usize> {
        ax_err!(InvalidInput)
    }

    fn write_at(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        ax_err!(InvalidInput)
    }

    fn fsync(&self) -> VfsResult {
        ax_err!(InvalidInput)
    }

    fn truncate(&self, _size: u64) -> VfsResult {
        ax_err!(InvalidInput)
    }

    // directory operations:

    fn lookup(self: Arc<Self>, _path: &str) -> VfsResult<VfsNodeRef> {
        ax_err!(NotADirectory)
    }

    fn create(&self, _path: &str, _ty: VfsNodeType) -> VfsResult<VfsNodeRef> {
        ax_err!(NotADirectory)
    }

    fn remove(&self, _path: &str) -> VfsResult {
        ax_err!(NotADirectory)
    }

    fn read_dir(&self, _start_idx: usize, _dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        ax_err!(NotADirectory)
    }
}

pub mod __priv {
    pub use alloc::sync::Arc;
    pub use axerrno::ax_err;
}

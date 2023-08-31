//! Virtual filesystem interfaces used by [ArceOS](https://github.com/rcore-os/arceos).
//!
//! A filesystem is a set of files and directories (symbol links are not
//! supported currently), collectively referred to as **nodes**, which are
//! conceptually similar to [inodes] in Linux. A file system needs to implement
//! the [`VfsOps`] trait, its files and directories need to implement the
//! [`VfsNodeOps`] trait.
//!
//! The [`VfsOps`] trait provides the following operations on a filesystem:
//!
//! - [`mount()`](VfsOps::mount): Do something when the filesystem is mounted.
//! - [`umount()`](VfsOps::umount): Do something when the filesystem is unmounted.
//! - [`format()`](VfsOps::format): Format the filesystem.
//! - [`statfs()`](VfsOps::statfs): Get the attributes of the filesystem.
//! - [`root_dir()`](VfsOps::root_dir): Get root directory of the filesystem.
//!
//! The [`VfsNodeOps`] trait provides the following operations on a file or a
//! directory:
//!
//! | Operation | Description | file/directory |
//! | --- | --- | --- |
//! | [`open()`](VfsNodeOps::open) | Do something when the node is opened | both |
//! | [`release()`](VfsNodeOps::release) | Do something when the node is closed | both |
//! | [`get_attr()`](VfsNodeOps::get_attr) | Get the attributes of the node | both |
//! | [`read_at()`](VfsNodeOps::read_at) | Read data from the file | file |
//! | [`write_at()`](VfsNodeOps::write_at) | Write data to the file | file |
//! | [`fsync()`](VfsNodeOps::fsync) | Synchronize the file data to disk | file |
//! | [`truncate()`](VfsNodeOps::truncate) | Truncate the file | file |
//! | [`parent()`](VfsNodeOps::parent) | Get the parent directory | directory |
//! | [`lookup()`](VfsNodeOps::lookup) | Lookup the node with the given path | directory |
//! | [`create()`](VfsNodeOps::create) | Create a new node with the given path | directory |
//! | [`remove()`](VfsNodeOps::remove) | Remove the node with the given path | directory |
//! | [`read_dir()`](VfsNodeOps::read_dir) | Read directory entries | directory |
//!
//! [inodes]: https://en.wikipedia.org/wiki/Inode

#![no_std]

extern crate alloc;

mod macros;
mod structs;

pub mod path;

use alloc::sync::Arc;
use axerrno::{ax_err, AxError, AxResult};

pub use self::structs::{FileSystemInfo, VfsDirEntry, VfsNodeAttr, VfsNodePerm, VfsNodeType};

/// A wrapper of [`Arc<dyn VfsNodeOps>`].
pub type VfsNodeRef = Arc<dyn VfsNodeOps>;

/// Alias of [`AxError`].
pub type VfsError = AxError;

/// Alias of [`AxResult`].
pub type VfsResult<T = ()> = AxResult<T>;

/// Filesystem operations.
pub trait VfsOps: Send + Sync {
    /// Do something when the filesystem is mounted.
    fn mount(&self, _path: &str, _mount_point: VfsNodeRef) -> VfsResult {
        Ok(())
    }

    /// Do something when the filesystem is unmounted.
    fn umount(&self) -> VfsResult {
        Ok(())
    }

    /// Format the filesystem.
    fn format(&self) -> VfsResult {
        ax_err!(Unsupported)
    }

    /// Get the attributes of the filesystem.
    fn statfs(&self) -> VfsResult<FileSystemInfo> {
        ax_err!(Unsupported)
    }

    /// Get the root directory of the filesystem.
    fn root_dir(&self) -> VfsNodeRef;
}

/// Node (file/directory) operations.
pub trait VfsNodeOps: Send + Sync {
    /// Do something when the node is opened.
    fn open(&self) -> VfsResult {
        Ok(())
    }

    /// Do something when the node is closed.
    fn release(&self) -> VfsResult {
        Ok(())
    }

    /// Get the attributes of the node.
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        ax_err!(Unsupported)
    }

    // file operations:

    /// Read data from the file at the given offset.
    fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> VfsResult<usize> {
        ax_err!(InvalidInput)
    }

    /// Write data to the file at the given offset.
    fn write_at(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        ax_err!(InvalidInput)
    }

    /// Flush the file, synchronize the data to disk.
    fn fsync(&self) -> VfsResult {
        ax_err!(InvalidInput)
    }

    /// Truncate the file to the given size.
    fn truncate(&self, _size: u64) -> VfsResult {
        ax_err!(InvalidInput)
    }

    // directory operations:

    /// Get the parent directory of this directory.
    ///
    /// Return `None` if the node is a file.
    fn parent(&self) -> Option<VfsNodeRef> {
        None
    }

    /// Lookup the node with given `path` in the directory.
    ///
    /// Return the node if found.
    fn lookup(self: Arc<Self>, _path: &str) -> VfsResult<VfsNodeRef> {
        ax_err!(Unsupported)
    }

    /// Create a new node with the given `path` in the directory
    ///
    /// Return [`Ok(())`](Ok) if it already exists.
    fn create(&self, _path: &str, _ty: VfsNodeType) -> VfsResult {
        ax_err!(Unsupported)
    }

    /// Remove the node with the given `path` in the directory.
    fn remove(&self, _path: &str) -> VfsResult {
        ax_err!(Unsupported)
    }

    /// Read directory entries into `dirents`, starting from `start_idx`.
    fn read_dir(&self, _start_idx: usize, _dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        ax_err!(Unsupported)
    }

    /// Renames or moves existing file or directory.
    fn rename(&self, _src_path: &str, _dst_path: &str) -> VfsResult {
        ax_err!(Unsupported)
    }

    /// Convert `&self` to [`&dyn Any`][1] that can use
    /// [`Any::downcast_ref`][2].
    ///
    /// [1]: core::any::Any
    /// [2]: core::any::Any#method.downcast_ref
    fn as_any(&self) -> &dyn core::any::Any {
        unimplemented!()
    }
}

#[doc(hidden)]
pub mod __priv {
    pub use alloc::sync::Arc;
    pub use axerrno::ax_err;
}

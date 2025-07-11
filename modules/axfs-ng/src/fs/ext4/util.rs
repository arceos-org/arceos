use axfs_ng_vfs::{NodeType, VfsError};
use lwext4_rust::{DummyHal, Ext4Error, InodeType};

use super::Ext4Disk;

pub type LwExt4Filesystem = lwext4_rust::Ext4Filesystem<DummyHal, Ext4Disk>;

pub fn into_vfs_err(err: Ext4Error) -> VfsError {
    VfsError::try_from(err.code).unwrap_or(VfsError::EIO)
}

pub fn into_vfs_type(ty: InodeType) -> NodeType {
    match ty {
        InodeType::RegularFile => NodeType::RegularFile,
        InodeType::Directory => NodeType::Directory,
        InodeType::CharacterDevice => NodeType::CharacterDevice,
        InodeType::BlockDevice => NodeType::BlockDevice,
        InodeType::Fifo => NodeType::Fifo,
        InodeType::Socket => NodeType::Socket,
        InodeType::Symlink => NodeType::Symlink,
        InodeType::Unknown => NodeType::Unknown,
    }
}

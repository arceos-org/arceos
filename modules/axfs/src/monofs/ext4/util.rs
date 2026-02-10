use axerrno::LinuxError;
use axfs_ng_vfs::{NodeType, VfsError};
use lwext4_rust::{Ext4Error, InodeType, SystemHal};

use super::Ext4Disk;

pub struct AxHal;
impl SystemHal for AxHal {
    fn now() -> Option<core::time::Duration> {
        if cfg!(feature = "times") {
            Some(axhal::time::wall_time())
        } else {
            None
        }
    }
}

pub type LwExt4Filesystem = lwext4_rust::Ext4Filesystem<AxHal, Ext4Disk>;

pub fn into_vfs_err(err: Ext4Error) -> VfsError {
    let linux_error = LinuxError::try_from(err.code).unwrap_or(LinuxError::EIO);
    VfsError::from(linux_error).canonicalize()
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

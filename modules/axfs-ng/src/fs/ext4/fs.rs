use alloc::sync::Arc;
use core::cell::OnceCell;

use axdriver::AxBlockDevice;
use axfs_ng_vfs::{
    DirEntry, DirNode, Filesystem, FilesystemOps, Reference, StatFs, VfsResult, path::MAX_NAME_LEN,
};
use lock_api::{Mutex, MutexGuard, RawMutex};
use lwext4_rust::ffi::EXT4_ROOT_INO;

use super::{
    Ext4Disk, Inode,
    util::{LwExt4Filesystem, into_vfs_err},
};

pub struct Ext4Filesystem<M> {
    inner: Mutex<M, LwExt4Filesystem>,
    root_dir: OnceCell<DirEntry<M>>,
}

impl<M: RawMutex> Ext4Filesystem<M> {
    pub fn new(dev: AxBlockDevice) -> VfsResult<Filesystem<M>>
    where
        M: Send + Sync + 'static,
    {
        let ext4 = lwext4_rust::Ext4Filesystem::new(Ext4Disk(dev)).map_err(into_vfs_err)?;

        let fs = Arc::new(Self {
            inner: Mutex::new(ext4),
            root_dir: OnceCell::new(),
        });
        let _ = fs.root_dir.set(DirEntry::new_dir(
            |this| DirNode::new(Inode::new(fs.clone(), EXT4_ROOT_INO, Some(this))),
            Reference::root(),
        ));
        Ok(Filesystem::new(fs))
    }

    pub(crate) fn lock(&self) -> MutexGuard<M, LwExt4Filesystem> {
        self.inner.lock()
    }
}

unsafe impl<M> Send for Ext4Filesystem<M> {}

unsafe impl<M> Sync for Ext4Filesystem<M> {}

impl<M: RawMutex + 'static> FilesystemOps<M> for Ext4Filesystem<M> {
    fn name(&self) -> &str {
        "ext4"
    }

    fn root_dir(&self) -> DirEntry<M> {
        self.root_dir.get().unwrap().clone()
    }

    fn stat(&self) -> VfsResult<StatFs> {
        let mut fs = self.lock();
        let stat = fs.stat().map_err(into_vfs_err)?;
        Ok(StatFs {
            fs_type: 0xef53,
            block_size: stat.block_size as _,
            blocks: stat.blocks_count,
            blocks_free: stat.free_blocks_count,
            blocks_available: stat.free_blocks_count,

            file_count: stat.inodes_count as _,
            free_file_count: stat.free_inodes_count as _,

            name_length: MAX_NAME_LEN as _,
            fragment_size: 0,
            mount_flags: 0,
        })
    }
}

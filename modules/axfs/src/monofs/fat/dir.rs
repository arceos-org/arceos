use alloc::{string::String, sync::Arc};
use core::{any::Any, mem, ops::Deref, time::Duration};

use axfs_ng_vfs::{
    DeviceId, DirEntry, DirEntrySink, DirNode, DirNodeOps, FilesystemOps, Metadata, MetadataUpdate,
    NodeFlags, NodeOps, NodePermission, NodeType, Reference, VfsError, VfsResult, WeakDirEntry,
};

use super::{
    FsRef, ff,
    file::FatFileNode,
    fs::FatFilesystem,
    util::{file_metadata, into_vfs_err},
};

pub struct FatDirNode {
    fs: Arc<FatFilesystem>,
    pub(crate) inner: FsRef<ff::Dir<'static>>,
    inode: u64,
    this: WeakDirEntry,
}

impl FatDirNode {
    pub fn new(fs: Arc<FatFilesystem>, dir: ff::Dir, inode: u64, this: WeakDirEntry) -> DirNode {
        DirNode::new(Arc::new(Self {
            fs,
            // SAFETY: FsRef guarantees correct lifetime
            inner: FsRef::new(unsafe { mem::transmute::<ff::Dir, ff::Dir>(dir) }),
            inode,
            this,
        }))
    }

    fn create_entry(&self, entry: ff::DirEntry, name: impl Into<String>, inode: u64) -> DirEntry {
        let reference = Reference::new(self.this.upgrade(), name.into());
        if entry.is_file() {
            DirEntry::new_file(
                FatFileNode::new(self.fs.clone(), entry.to_file(), inode),
                NodeType::RegularFile,
                reference,
            )
        } else {
            DirEntry::new_dir(
                |this| FatDirNode::new(self.fs.clone(), entry.to_dir(), inode, this),
                reference,
            )
        }
    }
}

unsafe impl Send for FatDirNode {}

unsafe impl Sync for FatDirNode {}

impl NodeOps for FatDirNode {
    fn inode(&self) -> u64 {
        self.inode
    }

    fn metadata(&self) -> VfsResult<Metadata> {
        let fs = self.fs.lock();
        let dir = self.inner.borrow(&fs);
        if let Some(file) = dir.as_file() {
            return Ok(file_metadata(&fs, file, NodeType::Directory));
        }

        // root directory
        let block_size = fs.inner.bytes_per_sector() as u64;
        Ok(Metadata {
            inode: self.inode(),
            device: 0,
            nlink: 1,
            mode: NodePermission::default(),
            node_type: NodeType::Directory,
            uid: 0,
            gid: 0,
            size: block_size,
            block_size,
            blocks: 1,
            rdev: DeviceId::default(),
            atime: Duration::default(),
            mtime: Duration::default(),
            ctime: Duration::default(),
        })
    }

    fn update_metadata(&self, _update: MetadataUpdate) -> VfsResult<()> {
        // TODO: update metadata on directory
        Ok(())
    }

    fn filesystem(&self) -> &dyn FilesystemOps {
        self.fs.deref()
    }

    fn sync(&self, _data_only: bool) -> VfsResult<()> {
        Ok(())
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }

    fn flags(&self) -> NodeFlags {
        NodeFlags::BLOCKING
    }
}

impl DirNodeOps for FatDirNode {
    fn read_dir(&self, offset: u64, sink: &mut dyn DirEntrySink) -> VfsResult<usize> {
        let mut fs = self.fs.lock();
        let dir = self.inner.borrow(&fs);
        let this_entry = self.this.upgrade().unwrap();
        let dir_node = this_entry.as_dir()?;

        let mut count = 0;
        for entry in dir.iter().skip(offset as usize) {
            let entry = entry.map_err(into_vfs_err)?;
            let name = entry.file_name().to_ascii_lowercase();
            let node_type = if entry.is_file() {
                NodeType::RegularFile
            } else {
                NodeType::Directory
            };
            let inode = if let Some(entry) = dir_node.lookup_cache(&name) {
                entry.inode()
            } else {
                let entry = self.create_entry(entry, name.clone(), fs.alloc_inode());
                let inode = entry.inode();
                dir_node.insert_cache(name.clone(), entry);
                inode
            };
            if !sink.accept(&name, inode, node_type, offset + count + 1) {
                break;
            }
            count += 1;
        }
        Ok(count as usize)
    }

    fn lookup(&self, name: &str) -> VfsResult<DirEntry> {
        let mut fs = self.fs.lock();
        let dir = self.inner.borrow(&fs);
        dir.iter()
            .find_map(|entry| entry.ok().filter(|it| it.eq_name(name)))
            .map(|entry| self.create_entry(entry, name.to_ascii_lowercase(), fs.alloc_inode()))
            .ok_or(VfsError::NotFound)
    }

    fn create(
        &self,
        name: &str,
        node_type: NodeType,
        _permission: NodePermission,
    ) -> VfsResult<DirEntry> {
        let mut fs = self.fs.lock();
        let dir = self.inner.borrow(&fs);
        let reference = Reference::new(self.this.upgrade(), name.to_ascii_lowercase());
        match node_type {
            NodeType::RegularFile => dir
                .create_file(name)
                .map(|file| {
                    DirEntry::new_file(
                        FatFileNode::new(self.fs.clone(), file, fs.alloc_inode()),
                        NodeType::RegularFile,
                        reference,
                    )
                })
                .map_err(into_vfs_err),
            NodeType::Directory => dir
                .create_dir(name)
                .map(|dir| {
                    DirEntry::new_dir(
                        |this| FatDirNode::new(self.fs.clone(), dir, fs.alloc_inode(), this),
                        reference,
                    )
                })
                .map_err(into_vfs_err),
            _ => Err(VfsError::InvalidInput),
        }
    }

    fn link(&self, _name: &str, _node: &DirEntry) -> VfsResult<DirEntry> {
        //  EPERM  The filesystem containing oldpath and newpath does not
        //         support the creation of hard links.
        Err(VfsError::PermissionDenied)
    }

    fn unlink(&self, name: &str) -> VfsResult<()> {
        let fs = self.fs.lock();
        let dir = self.inner.borrow(&fs);
        dir.remove(name).map_err(into_vfs_err)
    }

    fn rename(&self, src_name: &str, dst_dir: &DirNode, dst_name: &str) -> VfsResult<()> {
        let fs = self.fs.lock();
        let dst_dir: Arc<Self> = dst_dir.downcast().map_err(|_| VfsError::InvalidInput)?;

        let dir = self.inner.borrow(&fs);

        // The default implementation throws EEXIST if dst exists, so we need to
        // handle it
        match dst_dir.inner.borrow(&fs).remove(dst_name) {
            Ok(_) => {}
            Err(fatfs::Error::NotFound) => {}
            Err(err) => return Err(into_vfs_err(err)),
        }

        dir.rename(src_name, dst_dir.inner.borrow(&fs), dst_name)
            .map_err(into_vfs_err)
    }
}

impl Drop for FatDirNode {
    fn drop(&mut self) {
        self.fs.lock().release_inode(self.inode);
    }
}

use alloc::{sync::Arc, vec};
use core::{any::Any, mem, ops::Deref, task::Context};

use axfs_ng_vfs::{
    FileNode, FileNodeOps, FilesystemOps, Metadata, MetadataUpdate, NodeFlags, NodeOps, NodeType,
    VfsError, VfsResult,
};
use axpoll::{IoEvents, Pollable};
use fatfs::{Read, Seek, SeekFrom, Write};

use super::{
    FsRef, ff,
    fs::FatFilesystem,
    util::{file_metadata, into_vfs_err, update_file_metadata},
};
use crate::fs::fat::fs::FatFilesystemInner;

pub struct FatFileNode {
    fs: Arc<FatFilesystem>,
    inner: FsRef<ff::File<'static>>,
    inode: u64,
}

impl FatFileNode {
    pub fn new(fs: Arc<FatFilesystem>, file: ff::File, inode: u64) -> FileNode {
        FileNode::new(Arc::new(Self {
            fs,
            // SAFETY: FsRef guarantees correct lifetime
            inner: FsRef::new(unsafe { mem::transmute::<ff::File, ff::File>(file) }),
            inode,
        }))
    }
}

fn grow_file(fs: &FatFilesystemInner, file: &mut ff::File<'static>, len: u64) -> VfsResult<()> {
    // rust-fatfs does not support growing files directly. We need to
    // pad with zeros manually.
    let mut pos = file.seek(SeekFrom::End(0)).map_err(into_vfs_err)?;
    let block_size = fs.inner.bytes_per_sector() as usize;
    let block = vec![0; block_size];

    while pos < len {
        let write = (block_size - (pos as usize & (block_size - 1))).min((len - pos) as usize);
        file.write(&block[0..write]).map_err(into_vfs_err)?;
        pos += write as u64;
    }
    Ok(())
}

unsafe impl Send for FatFileNode {}

unsafe impl Sync for FatFileNode {}

impl NodeOps for FatFileNode {
    fn inode(&self) -> u64 {
        self.inode
    }

    fn metadata(&self) -> VfsResult<Metadata> {
        let fs = self.fs.lock();
        let file = self.inner.borrow(&fs);
        Ok(file_metadata(&fs, file, NodeType::RegularFile))
    }

    fn update_metadata(&self, update: MetadataUpdate) -> VfsResult<()> {
        // FatFS has no ownership & permission

        let fs = self.fs.lock();
        let file = self.inner.borrow_mut(&fs);
        update_file_metadata(file, update);
        Ok(())
    }

    fn filesystem(&self) -> &dyn FilesystemOps {
        self.fs.deref()
    }

    fn len(&self) -> VfsResult<u64> {
        let fs = self.fs.lock();
        let file = self.inner.borrow(&fs);
        Ok(file.size().unwrap_or(0) as u64)
    }

    fn sync(&self, _data_only: bool) -> VfsResult<()> {
        let fs = self.fs.lock();
        let file = self.inner.borrow_mut(&fs);
        file.flush().map_err(into_vfs_err)
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }

    fn flags(&self) -> NodeFlags {
        NodeFlags::BLOCKING
    }
}

impl FileNodeOps for FatFileNode {
    fn read_at(&self, mut buf: &mut [u8], offset: u64) -> VfsResult<usize> {
        let fs = self.fs.lock();
        let file = self.inner.borrow_mut(&fs);
        file.seek(SeekFrom::Start(offset)).map_err(into_vfs_err)?;

        let mut read = 0;
        loop {
            let n = file.read(buf).map_err(into_vfs_err)?;
            if n == 0 {
                return Ok(read);
            }
            read += n;
            buf = &mut buf[n..];
        }
    }

    fn write_at(&self, mut buf: &[u8], offset: u64) -> VfsResult<usize> {
        let fs = self.fs.lock();
        let file = self.inner.borrow_mut(&fs);
        if offset > file.size().unwrap_or(0) as u64 {
            grow_file(&fs, file, offset)?;
        }
        file.seek(SeekFrom::Start(offset)).map_err(into_vfs_err)?;

        let mut written = 0;
        loop {
            let n = file.write(buf).map_err(into_vfs_err)?;
            if n == 0 {
                return Ok(written);
            }
            written += n;
            buf = &buf[n..];
        }
    }

    fn append(&self, buf: &[u8]) -> VfsResult<(usize, u64)> {
        let fs = self.fs.lock();
        let file = self.inner.borrow_mut(&fs);
        file.seek(SeekFrom::End(0)).map_err(into_vfs_err)?;
        let written = file.write(buf).map_err(into_vfs_err)?;
        Ok((written, file.size().unwrap_or(0) as u64))
    }

    fn set_len(&self, len: u64) -> VfsResult<()> {
        let fs = self.fs.lock();
        let file = self.inner.borrow_mut(&fs);
        if len <= file.size().unwrap_or(0) as u64 {
            file.seek(SeekFrom::Start(len)).map_err(into_vfs_err)?;
            file.truncate().map_err(into_vfs_err)
        } else {
            grow_file(&fs, file, len)
        }
    }

    fn set_symlink(&self, _target: &str) -> VfsResult<()> {
        Err(VfsError::PermissionDenied)
    }
}

impl Pollable for FatFileNode {
    fn poll(&self) -> IoEvents {
        IoEvents::IN | IoEvents::OUT
    }

    fn register(&self, _context: &mut Context<'_>, _events: IoEvents) {}
}

impl Drop for FatFileNode {
    fn drop(&mut self) {
        self.fs.lock().release_inode(self.inode);
    }
}

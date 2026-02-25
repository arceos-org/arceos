// Copyright 2025 The Axvisor Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use axfs_vfs::{
    VfsDirEntry, VfsError, VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeRef, VfsNodeType, VfsOps,
    VfsResult,
};
use rsext4::{
    Ext4FileSystem as Rsext4FileSystem, Jbd2Dev,
    api::{OpenFile, fs_mount, lseek, open, read_at},
    dir::{get_inode_with_num, mkdir},
    entries::classic_dir::list_entries,
    error::{BlockDevError, BlockDevResult},
    file::{delete_dir, mkfile, mv, truncate, unlink, write_file},
    loopfile::resolve_inode_block_allextend,
};
use spin::Mutex;

use crate::dev::{Disk, Partition};

/// Block size for ext4 filesystem operations
pub const BLOCK_SIZE: usize = 4096;

/// Ext4 filesystem implementation that works with a disk device
#[allow(dead_code)]
pub struct Ext4FileSystem {
    inner: Arc<Mutex<Jbd2Dev<Disk>>>,
    fs: Arc<Mutex<Rsext4FileSystem>>,
}

/// Ext4FileSystem that works with a partition
pub struct Ext4FileSystemPartition {
    inner: Arc<Mutex<Jbd2Dev<Partition>>>,
    fs: Arc<Mutex<Rsext4FileSystem>>,
}

unsafe impl Sync for Ext4FileSystem {}
unsafe impl Send for Ext4FileSystem {}

unsafe impl Sync for Ext4FileSystemPartition {}
unsafe impl Send for Ext4FileSystemPartition {}

impl Ext4FileSystem {
    /// Create a new ext4 filesystem from a disk device
    #[allow(dead_code)]
    pub fn new(disk: Disk) -> Self {
        info!(
            "Got Disk size:{}, position:{}",
            disk.size(),
            disk.position()
        );
        let mut inner = Jbd2Dev::initial_jbd2dev(0, disk, false);
        let fs = fs_mount(&mut inner).expect("failed to initialize EXT4 filesystem");
        Self {
            inner: Arc::new(Mutex::new(inner)),
            fs: Arc::new(Mutex::new(fs)),
        }
    }

    /// Create a new ext4 filesystem from a partition
    pub fn from_partition(partition: Partition) -> Ext4FileSystemPartition {
        info!(
            "Got Partition size:{}, position:{}",
            partition.size(),
            partition.position()
        );
        let mut inner = Jbd2Dev::initial_jbd2dev(0, partition, false);
        let fs = fs_mount(&mut inner).expect("failed to initialize EXT4 filesystem on partition");
        Ext4FileSystemPartition {
            inner: Arc::new(Mutex::new(inner)),
            fs: Arc::new(Mutex::new(fs)),
        }
    }
}

/// The [`VfsOps`] trait provides operations on a filesystem.
impl VfsOps for Ext4FileSystem {
    fn root_dir(&self) -> VfsNodeRef {
        debug!("Get root_dir");
        Arc::new(FileWrapper::new(
            "/",
            Ext4Inner::Disk(Arc::clone(&self.inner)),
            Arc::clone(&self.fs),
        ))
    }
}

/// The [`VfsOps`] trait provides operations on a filesystem.
impl VfsOps for Ext4FileSystemPartition {
    fn root_dir(&self) -> VfsNodeRef {
        debug!("Get root_dir");
        Arc::new(FileWrapper::new(
            "/",
            Ext4Inner::Partition(Arc::clone(&self.inner)),
            Arc::clone(&self.fs),
        ))
    }
}

/// Inner state for ext4 filesystem, either backed by a full disk or a partition
#[derive(Clone)]
pub enum Ext4Inner {
    /// Full disk device
    Disk(Arc<Mutex<Jbd2Dev<Disk>>>),
    /// Partition device
    Partition(Arc<Mutex<Jbd2Dev<Partition>>>),
}

/// Wrapper for files and directories in the ext4 filesystem
pub struct FileWrapper {
    path: String,
    file: Mutex<Option<OpenFile>>,
    inner: Ext4Inner,
    fs: Arc<Mutex<Rsext4FileSystem>>,
}

unsafe impl Send for FileWrapper {}
unsafe impl Sync for FileWrapper {}

impl FileWrapper {
    fn new(path: &str, inner: Ext4Inner, fs: Arc<Mutex<Rsext4FileSystem>>) -> Self {
        debug!("FileWrapper new {}", path);
        Self {
            path: path.to_string(),
            file: Mutex::new(None),
            inner,
            fs,
        }
    }

    fn path_deal_with(&self, path: &str) -> String {
        if path.starts_with('/') {
            debug!("path_deal_with: {}", path);
        }
        let trim_path = path.trim_matches('/');
        if trim_path.is_empty() || trim_path == "." {
            return self.path.to_string();
        }

        if let Some(rest) = trim_path.strip_prefix("./") {
            //if starts with "./"
            return self.path_deal_with(rest);
        }
        let rest_p = trim_path.replace("//", "/");
        if trim_path != rest_p {
            return self.path_deal_with(&rest_p);
        }

        let base_path = self.path.trim_end_matches('/');
        if base_path == "/" {
            format!("/{}", trim_path)
        } else {
            format!("{}/{}", base_path, trim_path)
        }
    }
}

/// The [`VfsNodeOps`] trait provides operations on a file or a directory.
impl VfsNodeOps for FileWrapper {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        let mut fs = self.fs.lock();
        let perm = VfsNodePerm::from_bits_truncate(0o755);
        let (_inode_num, inode) = match self.inner {
            Ext4Inner::Disk(ref inner) => {
                let mut inner = inner.lock();
                get_inode_with_num(&mut *fs, &mut *inner, &self.path)
                    .map_err(|_| VfsError::Io)?
                    .ok_or(VfsError::NotFound)?
            }
            Ext4Inner::Partition(ref inner) => {
                let mut inner = inner.lock();
                get_inode_with_num(&mut *fs, &mut *inner, &self.path)
                    .map_err(|_| VfsError::Io)?
                    .ok_or(VfsError::NotFound)?
            }
        };
        let vtype = if inode.is_dir() {
            VfsNodeType::Dir
        } else {
            VfsNodeType::File
        };
        let size = inode.size() as u64;
        let blocks = inode.blocks_count() as u64;

        trace!(
            "get_attr of {:?}, size: {}, blocks: {}",
            self.path, size, blocks
        );

        Ok(VfsNodeAttr::new(perm, vtype, size, blocks))
    }

    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult {
        debug!("create {:?} on Ext4fs: {}", ty, path);
        let fpath = self.path_deal_with(path);
        if fpath.is_empty() {
            return Ok(());
        }

        let mut fs = self.fs.lock();
        match self.inner {
            Ext4Inner::Disk(ref inner) => {
                let mut inner = inner.lock();
                match ty {
                    VfsNodeType::Dir => {
                        mkdir(&mut *inner, &mut *fs, &fpath);
                    }
                    _ => {
                        mkfile(&mut *inner, &mut *fs, &fpath, None, None);
                    }
                }
            }
            Ext4Inner::Partition(ref inner) => {
                let mut inner = inner.lock();
                match ty {
                    VfsNodeType::Dir => {
                        mkdir(&mut *inner, &mut *fs, &fpath);
                    }
                    _ => {
                        mkfile(&mut *inner, &mut *fs, &fpath, None, None);
                    }
                }
            }
        }
        Ok(())
    }

    fn remove(&self, path: &str) -> VfsResult {
        debug!("remove ext4fs: {}", path);
        let fpath = self.path_deal_with(path);
        assert!(!fpath.is_empty()); // already check at `root.rs`

        let mut fs = self.fs.lock();
        let (_inode_num, inode) = match self.inner {
            Ext4Inner::Disk(ref inner) => {
                let mut inner = inner.lock();
                get_inode_with_num(&mut *fs, &mut *inner, &fpath)
                    .map_err(|_| VfsError::Io)?
                    .ok_or(VfsError::NotFound)?
            }
            Ext4Inner::Partition(ref inner) => {
                let mut inner = inner.lock();
                get_inode_with_num(&mut *fs, &mut *inner, &fpath)
                    .map_err(|_| VfsError::Io)?
                    .ok_or(VfsError::NotFound)?
            }
        };

        match self.inner {
            Ext4Inner::Disk(ref inner) => {
                let mut inner = inner.lock();
                if inode.is_dir() {
                    delete_dir(&mut *fs, &mut *inner, &fpath);
                } else {
                    unlink(&mut *fs, &mut *inner, &fpath);
                }
            }
            Ext4Inner::Partition(ref inner) => {
                let mut inner = inner.lock();
                if inode.is_dir() {
                    delete_dir(&mut *fs, &mut *inner, &fpath);
                } else {
                    unlink(&mut *fs, &mut *inner, &fpath);
                }
            }
        }
        Ok(())
    }

    /// Get the parent directory of this directory.
    /// Return `None` if the node is a file.
    fn parent(&self) -> Option<VfsNodeRef> {
        let path = &self.path;
        debug!("Get the parent dir of {}", path);
        let path = path.trim_end_matches('/').trim_end_matches(|c| c != '/');
        if !path.is_empty() {
            return Some(Arc::new(Self::new(
                path,
                self.inner.clone(),
                Arc::clone(&self.fs),
            )));
        }
        None
    }

    /// Read directory entries into `dirents`, starting from `start_idx`.
    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        let mut fs = self.fs.lock();
        let (_inode_num, mut inode) = match self.inner {
            Ext4Inner::Disk(ref inner) => {
                let mut inner = inner.lock();
                get_inode_with_num(&mut *fs, &mut *inner, &self.path)
                    .map_err(|_| VfsError::Io)?
                    .ok_or(VfsError::NotFound)?
            }
            Ext4Inner::Partition(ref inner) => {
                let mut inner = inner.lock();
                get_inode_with_num(&mut *fs, &mut *inner, &self.path)
                    .map_err(|_| VfsError::Io)?
                    .ok_or(VfsError::NotFound)?
            }
        };

        if !inode.is_dir() {
            return Err(VfsError::Unsupported);
        }

        let blocks = match self.inner {
            Ext4Inner::Disk(ref inner) => {
                let mut inner = inner.lock();
                resolve_inode_block_allextend(&mut *fs, &mut *inner, &mut inode)
                    .map_err(|_| VfsError::Io)?
            }
            Ext4Inner::Partition(ref inner) => {
                let mut inner = inner.lock();
                resolve_inode_block_allextend(&mut *fs, &mut *inner, &mut inode)
                    .map_err(|_| VfsError::Io)?
            }
        };

        let mut data = Vec::new();
        for (_, phys_block) in blocks {
            let cached = match self.inner {
                Ext4Inner::Disk(ref inner) => {
                    let mut inner = inner.lock();
                    fs.datablock_cache
                        .get_or_load(&mut *inner, phys_block)
                        .map_err(|_| VfsError::Io)?
                }
                Ext4Inner::Partition(ref inner) => {
                    let mut inner = inner.lock();
                    fs.datablock_cache
                        .get_or_load(&mut *inner, phys_block)
                        .map_err(|_| VfsError::Io)?
                }
            };
            data.extend_from_slice(&cached.data);
        }

        let entries = list_entries(&data);
        let mut unique = BTreeMap::new();
        for entry in entries {
            if let Some(name) = entry.name_str() {
                if name != "." && name != ".." {
                    unique.insert(name.to_string(), entry.file_type);
                }
            }
        }
        let unique_vec: Vec<_> = unique.into_iter().collect();
        let mut count = 0;
        for (name, file_type) in unique_vec.iter().skip(start_idx) {
            if count >= dirents.len() {
                break;
            }
            let ty = match *file_type {
                2 => VfsNodeType::Dir,
                _ => VfsNodeType::File,
            };
            dirents[count] = VfsDirEntry::new(name, ty);
            count += 1;
        }
        Ok(count)
    }

    /// Lookup the node with given `path` in the directory.
    /// Return the node if found.
    fn lookup(self: Arc<Self>, path: &str) -> VfsResult<VfsNodeRef> {
        trace!("lookup ext4fs: {}, {}", self.path, path);
        let fpath = self.path_deal_with(path);
        if fpath.is_empty() {
            return Ok(self.clone());
        }

        let mut fs = self.fs.lock();
        let exists = match self.inner {
            Ext4Inner::Disk(ref inner) => {
                let mut inner = inner.lock();
                get_inode_with_num(&mut *fs, &mut *inner, &fpath)
                    .map_err(|_| VfsError::Io)?
                    .is_some()
            }
            Ext4Inner::Partition(ref inner) => {
                let mut inner = inner.lock();
                get_inode_with_num(&mut *fs, &mut *inner, &fpath)
                    .map_err(|_| VfsError::Io)?
                    .is_some()
            }
        };

        if exists {
            Ok(Arc::new(Self::new(
                &fpath,
                self.inner.clone(),
                Arc::clone(&self.fs),
            )))
        } else {
            Err(VfsError::NotFound)
        }
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let mut file_guard = self.file.lock();
        if file_guard.is_none() {
            let mut fs = self.fs.lock();
            *file_guard = match self.inner {
                Ext4Inner::Disk(ref inner) => {
                    let mut inner = inner.lock();
                    open(&mut *inner, &mut *fs, &self.path, false).ok()
                }
                Ext4Inner::Partition(ref inner) => {
                    let mut inner = inner.lock();
                    open(&mut *inner, &mut *fs, &self.path, false).ok()
                }
            };
        }

        if let Some(ref mut file) = *file_guard {
            let mut fs = self.fs.lock();
            lseek(file, offset);
            let data = match self.inner {
                Ext4Inner::Disk(ref inner) => {
                    let mut inner = inner.lock();
                    read_at(&mut *inner, &mut *fs, file, buf.len()).map_err(|_| VfsError::Io)?
                }
                Ext4Inner::Partition(ref inner) => {
                    let mut inner = inner.lock();
                    read_at(&mut *inner, &mut *fs, file, buf.len()).map_err(|_| VfsError::Io)?
                }
            };
            let len = data.len().min(buf.len());
            buf[..len].copy_from_slice(&data[..len]);
            Ok(len)
        } else {
            Err(VfsError::NotFound)
        }
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let mut fs = self.fs.lock();
        match self.inner {
            Ext4Inner::Disk(ref inner) => {
                let mut inner = inner.lock();
                write_file(&mut *inner, &mut *fs, &self.path, offset, buf)
                    .map_err(|_| VfsError::Io)?;
            }
            Ext4Inner::Partition(ref inner) => {
                let mut inner = inner.lock();
                write_file(&mut *inner, &mut *fs, &self.path, offset, buf)
                    .map_err(|_| VfsError::Io)?;
            }
        };
        Ok(buf.len())
    }

    fn truncate(&self, size: u64) -> VfsResult {
        let mut fs = self.fs.lock();
        match self.inner {
            Ext4Inner::Disk(ref inner) => {
                let mut inner = inner.lock();
                let _ = truncate(&mut *inner, &mut *fs, &self.path, size);
            }
            Ext4Inner::Partition(ref inner) => {
                let mut inner = inner.lock();
                let _ = truncate(&mut *inner, &mut *fs, &self.path, size);
            }
        }
        Ok(())
    }

    fn rename(&self, src_path: &str, dst_path: &str) -> VfsResult {
        debug!("rename from {} to {}", src_path, dst_path);

        let src_fpath = self.path_deal_with(src_path);
        let dst_fpath = self.path_deal_with(dst_path);

        let mut fs = self.fs.lock();
        match self.inner {
            Ext4Inner::Disk(ref inner) => {
                let mut inner = inner.lock();
                let _ = mv(&mut *fs, &mut *inner, &src_fpath, &dst_fpath);
            }
            Ext4Inner::Partition(ref inner) => {
                let mut inner = inner.lock();
                let _ = mv(&mut *fs, &mut *inner, &src_fpath, &dst_fpath);
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self as &dyn core::any::Any
    }
}

impl Drop for FileWrapper {
    fn drop(&mut self) {
        debug!("Drop struct FileWrapper {:?}", self.path);
        // File will be automatically closed when OpenFile is dropped
    }
}

impl rsext4::BlockDevice for Disk {
    fn write(&mut self, buffer: &[u8], block_id: u32, count: u32) -> BlockDevResult<()> {
        // RVlwext4 uses 4096 byte blocks, but Disk uses 512 byte blocks
        self.set_position(block_id as u64 * BLOCK_SIZE as u64);
        let mut total_written = 0;
        let to_write = count as usize * BLOCK_SIZE;

        while total_written < to_write {
            let remaining = &buffer[total_written..];
            let written = self
                .write_one(remaining)
                .map_err(|_| BlockDevError::WriteError)?;
            total_written += written;
        }

        Ok(())
    }

    fn read(&mut self, buffer: &mut [u8], block_id: u32, count: u32) -> BlockDevResult<()> {
        self.set_position(block_id as u64 * BLOCK_SIZE as u64);
        let mut total_read = 0;
        let to_read = count as usize * BLOCK_SIZE;

        while total_read < to_read {
            let remaining = &mut buffer[total_read..];
            let read = self
                .read_one(remaining)
                .map_err(|_| BlockDevError::ReadError)?;
            total_read += read;
        }

        Ok(())
    }

    fn open(&mut self) -> BlockDevResult<()> {
        Ok(())
    }

    fn close(&mut self) -> BlockDevResult<()> {
        Ok(())
    }

    fn total_blocks(&self) -> u64 {
        // RVlwext4 uses 4096 byte blocks
        self.size() / BLOCK_SIZE as u64
    }
}

impl rsext4::BlockDevice for Partition {
    fn write(&mut self, buffer: &[u8], block_id: u32, count: u32) -> BlockDevResult<()> {
        self.set_position(block_id as u64 * BLOCK_SIZE as u64);
        let mut total_written = 0;
        let to_write = count as usize * BLOCK_SIZE;

        while total_written < to_write {
            let remaining = &buffer[total_written..];
            let written = self
                .write_one(remaining)
                .map_err(|_| BlockDevError::WriteError)?;
            total_written += written;
        }

        Ok(())
    }

    fn read(&mut self, buffer: &mut [u8], block_id: u32, count: u32) -> BlockDevResult<()> {
        self.set_position(block_id as u64 * BLOCK_SIZE as u64);
        let mut total_read = 0;
        let to_read = count as usize * BLOCK_SIZE;

        while total_read < to_read {
            let remaining = &mut buffer[total_read..];
            let read = self
                .read_one(remaining)
                .map_err(|_| BlockDevError::ReadError)?;
            total_read += read;
        }

        Ok(())
    }

    fn open(&mut self) -> BlockDevResult<()> {
        Ok(())
    }

    fn close(&mut self) -> BlockDevResult<()> {
        Ok(())
    }

    fn total_blocks(&self) -> u64 {
        // RVlwext4 uses 4096 byte blocks
        self.size() / BLOCK_SIZE as u64
    }
}

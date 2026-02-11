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

//! FAT filesystem implementation

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::cell::OnceCell;

use axfatfs::{Dir, File, LossyOemCpConverter, NullTimeProvider, Read, Seek, SeekFrom, Write};
use axfs_vfs::{VfsDirEntry, VfsError, VfsNodePerm, VfsResult};
use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType, VfsOps};
use spin::Mutex;

use crate::dev::{Disk, Partition};

const BLOCK_SIZE: usize = 512;

/// FAT filesystem implementation
pub struct FatFileSystem {
    inner: axfatfs::FileSystem<PartitionWrapper, NullTimeProvider, LossyOemCpConverter>,
    root_dir: OnceCell<VfsNodeRef>,
}

/// A wrapper for Partition to implement the required traits for axfatfs
pub struct PartitionWrapper {
    partition: Partition,
}

impl PartitionWrapper {
    /// Creates a new partition wrapper
    pub fn new(partition: Partition) -> Self {
        Self { partition }
    }
}

impl axfatfs::IoBase for PartitionWrapper {
    type Error = ();
}

impl axfatfs::Read for PartitionWrapper {
    fn read(&mut self, mut buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut read_len = 0;
        while !buf.is_empty() {
            match self.partition.read_one(buf) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                    read_len += n;
                }
                Err(_) => return Err(()),
            }
        }
        Ok(read_len)
    }
}

impl axfatfs::Write for PartitionWrapper {
    fn write(&mut self, mut buf: &[u8]) -> Result<usize, Self::Error> {
        let mut write_len = 0;
        while !buf.is_empty() {
            match self.partition.write_one(buf) {
                Ok(0) => break,
                Ok(n) => {
                    buf = &buf[n..];
                    write_len += n;
                }
                Err(_) => return Err(()),
            }
        }
        Ok(write_len)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl axfatfs::Seek for PartitionWrapper {
    fn seek(&mut self, pos: axfatfs::SeekFrom) -> Result<u64, Self::Error> {
        let size = self.partition.size();
        let new_pos = match pos {
            axfatfs::SeekFrom::Start(pos) => Some(pos),
            axfatfs::SeekFrom::Current(off) => self.partition.position().checked_add_signed(off),
            axfatfs::SeekFrom::End(off) => size.checked_add_signed(off),
        }
        .ok_or(())?;
        if new_pos > size {
            warn!("Seek beyond the end of the partition");
        }
        self.partition.set_position(new_pos);
        Ok(new_pos)
    }
}

/// Wrapper for FAT file
pub struct FileWrapper<'a>(
    Mutex<File<'a, PartitionWrapper, NullTimeProvider, LossyOemCpConverter>>,
);
/// Wrapper for FAT directory
pub struct DirWrapper<'a>(Dir<'a, PartitionWrapper, NullTimeProvider, LossyOemCpConverter>);

unsafe impl Sync for FatFileSystem {}
unsafe impl Send for FatFileSystem {}
unsafe impl Send for FileWrapper<'_> {}
unsafe impl Sync for FileWrapper<'_> {}
unsafe impl Send for DirWrapper<'_> {}
unsafe impl Sync for DirWrapper<'_> {}

impl FatFileSystem {
    /// Creates a new FAT filesystem from a disk
    #[allow(dead_code)]
    pub fn new(disk: Disk) -> Self {
        let disk_size = disk.size();
        let wrapper = PartitionWrapper::new(crate::dev::Partition::new(disk, 0, disk_size / 512));
        let inner = axfatfs::FileSystem::new(wrapper, axfatfs::FsOptions::new())
            .expect("failed to initialize FAT filesystem");
        Self {
            inner,
            root_dir: OnceCell::new(),
        }
    }

    /// Create a new FAT filesystem from a partition
    pub fn from_partition(partition: Partition) -> Self {
        let wrapper = PartitionWrapper::new(partition);
        let inner = axfatfs::FileSystem::new(wrapper, axfatfs::FsOptions::new())
            .expect("failed to initialize FAT filesystem on partition");
        Self {
            inner,
            root_dir: OnceCell::new(),
        }
    }

    /// Initializes the FAT filesystem
    #[allow(dead_code)]
    pub fn init(&'static self) {
        // root_dir is already initialized in new(), so nothing to do here
    }

    fn new_file(
        file: File<'_, PartitionWrapper, NullTimeProvider, LossyOemCpConverter>,
    ) -> VfsNodeRef {
        // Use a Box to extend the lifetime of the file
        let file_box = Box::new(file);
        let file_static = unsafe {
            core::mem::transmute::<
                Box<File<'_, PartitionWrapper, NullTimeProvider, LossyOemCpConverter>>,
                Box<File<'static, PartitionWrapper, NullTimeProvider, LossyOemCpConverter>>,
            >(file_box)
        };
        let file_wrapper = FileWrapper(Mutex::new(*file_static));
        Arc::new(file_wrapper) as VfsNodeRef
    }

    fn new_dir(
        dir: Dir<'_, PartitionWrapper, NullTimeProvider, LossyOemCpConverter>,
    ) -> VfsNodeRef {
        // Use a Box to extend the lifetime of the dir
        let dir_box = Box::new(dir);
        let dir_static = unsafe {
            core::mem::transmute::<
                Box<Dir<'_, PartitionWrapper, NullTimeProvider, LossyOemCpConverter>>,
                Box<Dir<'static, PartitionWrapper, NullTimeProvider, LossyOemCpConverter>>,
            >(dir_box)
        };
        let dir_wrapper = DirWrapper(*dir_static);
        Arc::new(dir_wrapper) as VfsNodeRef
    }
}

impl VfsNodeOps for FileWrapper<'static> {
    axfs_vfs::impl_vfs_non_dir_default! {}

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        let size = self.0.lock().seek(SeekFrom::End(0)).map_err(as_vfs_err)?;
        let blocks = size.div_ceil(BLOCK_SIZE as u64);
        // FAT fs doesn't support permissions, we just set everything to 755
        let perm = VfsNodePerm::from_bits_truncate(0o755);
        Ok(VfsNodeAttr::new(perm, VfsNodeType::File, size, blocks))
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let mut file = self.0.lock();
        file.seek(SeekFrom::Start(offset)).map_err(as_vfs_err)?; // TODO: more efficient
        file.read(buf).map_err(as_vfs_err)
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let mut file = self.0.lock();
        file.seek(SeekFrom::Start(offset)).map_err(as_vfs_err)?; // TODO: more efficient
        file.write(buf).map_err(as_vfs_err)
    }

    fn truncate(&self, size: u64) -> VfsResult {
        let mut file = self.0.lock();
        let current_size = file.seek(SeekFrom::End(0)).map_err(as_vfs_err)?;

        if size <= current_size {
            // If the target size is smaller than the current size,
            // perform a standard truncation operation
            file.seek(SeekFrom::Start(size)).map_err(as_vfs_err)?; // TODO: more efficient
            file.truncate().map_err(as_vfs_err)
        } else {
            // Calculate the number of bytes to fill
            let mut zeros_needed = size - current_size;
            // Create a buffer of zeros
            let zeros = [0u8; 4096];
            while zeros_needed > 0 {
                let to_write = core::cmp::min(zeros_needed, zeros.len() as u64);
                let write_buf = &zeros[..to_write as usize];
                file.write(write_buf).map_err(as_vfs_err)?;
                zeros_needed -= to_write;
            }
            Ok(())
        }
    }
}

impl VfsNodeOps for DirWrapper<'static> {
    axfs_vfs::impl_vfs_dir_default! {}

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        // FAT fs doesn't support permissions, we just set everything to 755
        Ok(VfsNodeAttr::new(
            VfsNodePerm::from_bits_truncate(0o755),
            VfsNodeType::Dir,
            BLOCK_SIZE as u64,
            1,
        ))
    }

    fn parent(&self) -> Option<VfsNodeRef> {
        self.0
            .open_dir("..")
            .map_or(None, |dir| Some(FatFileSystem::new_dir(dir)))
    }

    fn lookup(self: Arc<Self>, path: &str) -> VfsResult<VfsNodeRef> {
        debug!("lookup at axfatfs: {}", path);
        let path = path.trim_matches('/');
        if path.is_empty() || path == "." {
            return Ok(self.clone());
        }
        if let Some(rest) = path.strip_prefix("./") {
            return self.lookup(rest);
        }

        // TODO: use `axfatfs::Dir::find_entry`, but it's not public.
        if let Ok(file) = self.0.open_file(path) {
            Ok(FatFileSystem::new_file(file))
        } else if let Ok(dir) = self.0.open_dir(path) {
            Ok(FatFileSystem::new_dir(dir))
        } else {
            Err(VfsError::NotFound)
        }
    }

    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult {
        debug!("create {:?} at axfatfs: {}", ty, path);
        let path = path.trim_matches('/');
        if path.is_empty() || path == "." {
            return Ok(());
        }
        if let Some(rest) = path.strip_prefix("./") {
            return self.create(rest, ty);
        }

        match ty {
            VfsNodeType::File => {
                self.0.create_file(path).map_err(as_vfs_err)?;
                Ok(())
            }
            VfsNodeType::Dir => {
                self.0.create_dir(path).map_err(as_vfs_err)?;
                Ok(())
            }
            _ => Err(VfsError::Unsupported),
        }
    }

    fn remove(&self, path: &str) -> VfsResult {
        debug!("remove at axfatfs: {}", path);
        let path = path.trim_matches('/');
        assert!(!path.is_empty()); // already check at `root.rs`
        if let Some(rest) = path.strip_prefix("./") {
            return self.remove(rest);
        }
        self.0.remove(path).map_err(as_vfs_err)
    }

    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        let mut iter = self.0.iter().skip(start_idx);
        for (i, out_entry) in dirents.iter_mut().enumerate() {
            let x = iter.next();
            match x {
                Some(Ok(entry)) => {
                    let ty = if entry.is_dir() {
                        VfsNodeType::Dir
                    } else if entry.is_file() {
                        VfsNodeType::File
                    } else {
                        unreachable!()
                    };
                    *out_entry = VfsDirEntry::new(&entry.file_name(), ty);
                }
                _ => return Ok(i),
            }
        }
        Ok(dirents.len())
    }

    fn rename(&self, src_path: &str, dst_path: &str) -> VfsResult {
        // `src_path` and `dst_path` should in the same mounted fs
        debug!(
            "rename at axfatfs, src_path: {}, dst_path: {}",
            src_path, dst_path
        );

        self.0
            .rename(src_path, &self.0, dst_path)
            .map_err(as_vfs_err)
    }
}

impl VfsOps for FatFileSystem {
    fn root_dir(&self) -> VfsNodeRef {
        self.root_dir
            .get_or_init(|| {
                debug!("Creating root directory for FAT filesystem");
                let root_dir = self.inner.root_dir();
                debug!("Successfully got root directory from FAT filesystem");
                Self::new_dir(root_dir)
            })
            .clone()
    }
}

impl axfatfs::IoBase for Disk {
    type Error = ();
}

impl Read for Disk {
    fn read(&mut self, mut buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut read_len = 0;
        while !buf.is_empty() {
            match self.read_one(buf) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                    read_len += n;
                }
                Err(_) => return Err(()),
            }
        }
        Ok(read_len)
    }
}

impl Write for Disk {
    fn write(&mut self, mut buf: &[u8]) -> Result<usize, Self::Error> {
        let mut write_len = 0;
        while !buf.is_empty() {
            match self.write_one(buf) {
                Ok(0) => break,
                Ok(n) => {
                    buf = &buf[n..];
                    write_len += n;
                }
                Err(_) => return Err(()),
            }
        }
        Ok(write_len)
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Seek for Disk {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let size = self.size();
        let new_pos = match pos {
            SeekFrom::Start(pos) => Some(pos),
            SeekFrom::Current(off) => self.position().checked_add_signed(off),
            SeekFrom::End(off) => size.checked_add_signed(off),
        }
        .ok_or(())?;
        if new_pos > size {
            warn!("Seek beyond the end of the block device");
        }
        self.set_position(new_pos);
        Ok(new_pos)
    }
}

const fn as_vfs_err(err: axfatfs::Error<()>) -> VfsError {
    use axfatfs::Error::*;
    match err {
        AlreadyExists => VfsError::AlreadyExists,
        CorruptedFileSystem => VfsError::InvalidData,
        DirectoryIsNotEmpty => VfsError::DirectoryNotEmpty,
        InvalidInput | InvalidFileNameLength | UnsupportedFileNameCharacter => {
            VfsError::InvalidInput
        }
        NotEnoughSpace => VfsError::StorageFull,
        NotFound => VfsError::NotFound,
        UnexpectedEof => VfsError::UnexpectedEof,
        WriteZero => VfsError::WriteZero,
        Io(_) => VfsError::Io,
        _ => VfsError::Io,
    }
}

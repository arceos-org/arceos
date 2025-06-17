use crate::alloc::string::{String, ToString};
use crate::dev::Disk;
use alloc::sync::Arc;
pub use axdriver_block::DevError;
use axerrno::AxError;
use axfs_vfs::{
    VfsDirEntry, VfsError, VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeRef, VfsNodeType, VfsOps,
    VfsResult,
};
use axsync::Mutex;
use lwext4_rust::bindings::{
    O_CREAT, O_RDONLY, O_RDWR, O_TRUNC, O_WRONLY, SEEK_CUR, SEEK_END, SEEK_SET,
};
use lwext4_rust::{Ext4BlockWrapper, Ext4File, InodeTypes, KernelDevOp};

pub const BLOCK_SIZE: usize = 512;

#[allow(dead_code)]
pub struct Ext4FileSystem {
    inner: Ext4BlockWrapper<Disk>,
    root: VfsNodeRef,
}

unsafe impl Sync for Ext4FileSystem {}
unsafe impl Send for Ext4FileSystem {}

impl Ext4FileSystem {
    #[cfg(feature = "use-ramdisk")]
    pub fn new(mut disk: Disk) -> Self {
        unimplemented!()
    }

    #[cfg(not(feature = "use-ramdisk"))]
    pub fn new(disk: Disk) -> Self {
        info!(
            "Got Disk size:{}, position:{}",
            disk.size(),
            disk.position()
        );
        let inner =
            Ext4BlockWrapper::<Disk>::new(disk).expect("failed to initialize EXT4 filesystem");
        let root = Arc::new(FileWrapper::new("/", InodeTypes::EXT4_DE_DIR));
        Self { inner, root }
    }
}

impl VfsOps for Ext4FileSystem {
    fn root_dir(&self) -> VfsNodeRef {
        debug!("Get root_dir");
        Arc::clone(&self.root)
    }
}

pub struct FileWrapper(Mutex<Ext4File>);

unsafe impl Send for FileWrapper {}
unsafe impl Sync for FileWrapper {}

impl FileWrapper {
    fn new(path: &str, types: InodeTypes) -> Self {
        info!("FileWrapper new {:?} {}", types, path);
        Self(Mutex::new(Ext4File::new(path, types)))
    }

    fn path_deal_with(&self, path: &str) -> String {
        let trim_path = path.trim_matches('/');
        if trim_path.is_empty() || trim_path == "." {
            return String::new();
        }

        let mut result = if let Some(rest) = trim_path.strip_prefix("./") {
            rest
        } else {
            trim_path
        }
        .replace("//", "/");

        if trim_path != result {
            return self.path_deal_with(&result);
        }

        let file = self.0.lock();
        let base_path = file.get_path().to_str().unwrap().trim_end_matches('/');
        let fpath = format!("{}/{}", base_path, trim_path);
        debug!("dealt with full path: {}", fpath);
        fpath
    }
}

impl VfsNodeOps for FileWrapper {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        let mut file = self.0.lock();
        let perm =
            VfsNodePerm::from_bits_truncate(file.file_mode_get().unwrap_or(0o755) as u16 & 0o777);
        let vtype = match file.file_type_get() {
            InodeTypes::EXT4_INODE_MODE_FIFO => VfsNodeType::Fifo,
            InodeTypes::EXT4_INODE_MODE_CHARDEV => VfsNodeType::CharDevice,
            InodeTypes::EXT4_INODE_MODE_DIRECTORY => VfsNodeType::Dir,
            InodeTypes::EXT4_INODE_MODE_BLOCKDEV => VfsNodeType::BlockDevice,
            InodeTypes::EXT4_INODE_MODE_FILE => VfsNodeType::File,
            InodeTypes::EXT4_INODE_MODE_SOFTLINK => VfsNodeType::SymLink,
            InodeTypes::EXT4_INODE_MODE_SOCKET => VfsNodeType::Socket,
            _ => {
                warn!("unknown file type");
                VfsNodeType::File
            }
        };

        let size = if vtype == VfsNodeType::File {
            let path = file.get_path().to_str().unwrap().to_string();
            file.file_open(&path, O_RDONLY)
                .map_err(|e| AxError::from(e as i32))?;
            let fsize = file.file_size();
            file.file_close().expect("failed to close fd");
            fsize
        } else {
            0
        };

        let blocks = (size + (BLOCK_SIZE as u64 - 1)) / BLOCK_SIZE as u64;
        trace!(
            "get_attr of {:?} {:?}, size: {}, blocks: {}",
            vtype,
            file.get_path(),
            size,
            blocks
        );

        Ok(VfsNodeAttr::new(perm, vtype, size, blocks))
    }

    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult {
        debug!("create {:?} on Ext4fs: {}", ty, path);
        let fpath = self.path_deal_with(path);
        if fpath.is_empty() {
            return Ok(());
        }

        let types = match ty {
            VfsNodeType::Fifo => InodeTypes::EXT4_DE_FIFO,
            VfsNodeType::CharDevice => InodeTypes::EXT4_DE_CHRDEV,
            VfsNodeType::Dir => InodeTypes::EXT4_DE_DIR,
            VfsNodeType::BlockDevice => InodeTypes::EXT4_DE_BLKDEV,
            VfsNodeType::File => InodeTypes::EXT4_DE_REG_FILE,
            VfsNodeType::SymLink => InodeTypes::EXT4_DE_SYMLINK,
            VfsNodeType::Socket => InodeTypes::EXT4_DE_SOCK,
        };

        let mut file = self.0.lock();
        if file.check_inode_exist(&fpath, types.clone()) {
            Ok(())
        } else if types == InodeTypes::EXT4_DE_DIR {
            file.dir_mk(&fpath)
                .map_err(|e| AxError::from(e as i32))?
                .then_some(())
        } else {
            file.file_open(&fpath, O_WRONLY | O_CREAT | O_TRUNC)
                .map_err(|e| AxError::from(e as i32))?;
            file.file_close().map_err(|e| AxError::from(e as i32))?
        }
    }

    fn remove(&self, path: &str) -> VfsResult {
        debug!("remove ext4fs: {}", path);
        let fpath = self.path_deal_with(path);
        let fpath = fpath.as_str();
        assert!(!fpath.is_empty());

        let mut file = self.0.lock();
        if file.check_inode_exist(fpath, InodeTypes::EXT4_DE_DIR) {
            file.dir_rm(fpath).map_err(|e| AxError::from(e as i32))?
        } else {
            file.file_remove(fpath)
                .map_err(|e| AxError::from(e as i32))?
        }
        .then_some(())
    }

    fn parent(&self) -> Option<VfsNodeRef> {
        let file = self.0.lock();
        if file.get_type() != InodeTypes::EXT4_DE_DIR {
            return None;
        }

        let path = file.get_path().to_str().unwrap();
        debug!("Get the parent dir of {}", path);
        let trimmed = path.trim_end_matches('/');
        if let Some(idx) = trimmed.rfind('/') {
            let parent_path = &trimmed[..idx];
            if !parent_path.is_empty() {
                return Some(Arc::new(Self::new(parent_path, InodeTypes::EXT4_DE_DIR)));
            }
        }
        None
    }

    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        let file = self.0.lock();
        let (names, inode_types) = file.lwext4_dir_entries().unwrap();
        for (i, out_entry) in dirents.iter_mut().enumerate() {
            let idx = start_idx + i;
            let (name, t) = match (names.get(idx), inode_types.get(idx)) {
                (Some(n), Some(t)) => (n, t),
                _ => return Ok(i),
            };

            let ty = match t {
                InodeTypes::EXT4_DE_DIR => VfsNodeType::Dir,
                InodeTypes::EXT4_DE_REG_FILE => VfsNodeType::File,
                InodeTypes::EXT4_DE_SYMLINK => VfsNodeType::SymLink,
                _ => {
                    error!("unknown file type: {:?}", t);
                    unreachable!()
                }
            };
            *out_entry = VfsDirEntry::new(core::str::from_utf8(name).unwrap(), ty);
        }
        Ok(dirents.len())
    }

    fn lookup(self: Arc<Self>, path: &str) -> VfsResult<VfsNodeRef> {
        trace!("lookup ext4fs: {:?}, {}", self.0.lock().get_path(), path);
        let fpath = self.path_deal_with(path);
        if fpath.is_empty() {
            return Ok(self.clone());
        }

        let mut file = self.0.lock();
        if file.check_inode_exist(&fpath, InodeTypes::EXT4_DE_DIR) {
            trace!("lookup new DIR FileWrapper");
            Ok(Arc::new(Self::new(&fpath, InodeTypes::EXT4_DE_DIR)))
        } else if file.check_inode_exist(&fpath, InodeTypes::EXT4_DE_REG_FILE) {
            trace!("lookup new FILE FileWrapper");
            Ok(Arc::new(Self::new(&fpath, InodeTypes::EXT4_DE_REG_FILE)))
        } else {
            Err(VfsError::NotFound)
        }
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        trace!("To read_at {}, buf len={}", offset, buf.len());
        let mut file = self.0.lock();
        let path = file.get_path().to_str().unwrap().to_string();
        file.file_open(&path, O_RDONLY)
            .map_err(|e| AxError::from(e as i32))?;
        file.file_seek(offset as i64, SEEK_SET)
            .map_err(|e| AxError::from(e as i32))?;
        let result = file.file_read(buf);
        file.file_close().expect("failed to close fd");
        result.map_err(|e| AxError::from(e as i32).into())
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        trace!("To write_at {}, buf len={}", offset, buf.len());
        let mut file = self.0.lock();
        let path = file.get_path().to_str().unwrap().to_string();
        file.file_open(&path, O_RDWR)
            .map_err(|e| AxError::from(e as i32))?;
        file.file_seek(offset as i64, SEEK_SET)
            .map_err(|e| AxError::from(e as i32))?;
        let result = file.file_write(buf);
        file.file_close().expect("failed to close fd");
        result.map_err(|e| AxError::from(e as i32).into())
    }

    fn truncate(&self, size: u64) -> VfsResult {
        debug!("truncate file to size={}", size);
        let mut file = self.0.lock();
        let path = file.get_path().to_str().unwrap().to_string();
        file.file_open(&path, O_RDWR | O_CREAT | O_TRUNC)
            .map_err(|e| AxError::from(e as i32))?;
        file.file_truncate(size)
            .map_err(|e| AxError::from(e as i32))?;
        file.file_close().expect("failed to close fd");
        Ok(())
    }

    fn rename(&self, src_path: &str, dst_path: &str) -> VfsResult {
        debug!("rename from {} to {}", src_path, dst_path);
        let mut file = self.0.lock();
        file.file_rename(src_path, dst_path)
            .map(|_| ())
            .map_err(|e| AxError::from(e as i32).into())
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl Drop for FileWrapper {
    fn drop(&mut self) {
        let mut file = self.0.lock();
        debug!("Drop struct FileWrapper {:?}", file.get_path());
        file.file_close().expect("failed to close fd");
        drop(file); // todo
    }
}

impl KernelDevOp for Disk {
    type DevType = Disk;

    fn read(dev: &mut Disk, mut buf: &mut [u8]) -> Result<usize, i32> {
        trace!("READ block device buf={}", buf.len());
        let mut read_len = 0;
        while !buf.is_empty() {
            match dev.read_one(buf) {
                Ok(0) => break,
                Ok(n) => {
                    buf = &mut buf[n..];
                    read_len += n;
                }
                Err(_) => return Err(DevError::Io as i32),
            }
        }
        trace!("READ rt len={}", read_len);
        Ok(read_len)
    }

    fn write(dev: &mut Self::DevType, mut buf: &[u8]) -> Result<usize, i32> {
        trace!("WRITE block device buf={}", buf.len());
        let mut write_len = 0;
        while !buf.is_empty() {
            match dev.write_one(buf) {
                Ok(0) => break,
                Ok(n) => {
                    buf = &buf[n..];
                    write_len += n;
                }
                Err(_) => return Err(DevError::Io as i32),
            }
        }
        trace!("WRITE rt len={}", write_len);
        Ok(write_len)
    }

    fn flush(_dev: &mut Self::DevType) -> Result<usize, i32> {
        debug!("uncomplicated");
        Ok(0)
    }

    fn seek(dev: &mut Disk, off: i64, whence: i32) -> Result<i64, i32> {
        let size = dev.size();
        let pos = dev.position();
        trace!(
            "SEEK block device size:{}, pos:{}, offset={}, whence={}",
            size, pos, off, whence
        );

        let new_pos = match whence as u32 {
            SEEK_SET => off,
            SEEK_CUR => pos as i64 + off,
            SEEK_END => size as i64 + off,
            _ => {
                error!("invalid seek() whence: {}", whence);
                return Err(DevError::Io as i32);
            }
        };

        if new_pos < 0 {
            warn!("Negative seek position");
            return Err(DevError::Io as i32);
        }

        if new_pos as u64 > size {
            warn!("Seek position is beyond device size");
        }
        dev.seek_position(new_pos as u64);
        Ok(new_pos)
    }
}

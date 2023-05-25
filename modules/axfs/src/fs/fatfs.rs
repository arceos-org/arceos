//! fatfs模块定义了FAT32文件系统相关的结构和实现。它基于fatfs库实现axfs_vfs的抽象,
//! 使ArceOS可以使用FAT32文件系统

use alloc::sync::Arc;
use core::cell::UnsafeCell;

use axfs_vfs::{VfsDirEntry, VfsError, VfsNodePerm, VfsResult};
use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType, VfsOps};
use axsync::Mutex;
use fatfs::{Dir, File, LossyOemCpConverter, NullTimeProvider, Read, Seek, SeekFrom, Write};
// LossyOemCpConverter表示OEM代码页到UTF-16的不可逆转换。这用于fatfs库将FAT32文件系统中的文件名从OEM代码页转换为UTF-16。
// NullTimeProvider表示不提供时间相关信息。fatfs库在某些操作中需要时间相关信息,此处通过NullTimeProvider表示不提供。
// Read、Write和Seek trait分别表示读、写和寻址操作。fatfs库通过这些trait来对抽象的Io设备进行操作。


use crate::dev::Disk;

const BLOCK_SIZE: usize = 512;

/// FAT文件系统结构体,封装fatfs库的FileSystem。
pub struct FatFileSystem {
    // fatfs库的FileSystem
    inner: fatfs::FileSystem<Disk, NullTimeProvider, LossyOemCpConverter>,
    // FAT文件系统的根目录
    root_dir: UnsafeCell<Option<VfsNodeRef>>,
}

/// 封装fatfs库的File,实现Send和Sync以用于多线程环境。
pub struct FileWrapper<'a>(Mutex<File<'a, Disk, NullTimeProvider, LossyOemCpConverter>>);

/// 封装fatfs库的Dir,实现Send和Sync以用于多线程环境。
pub struct DirWrapper<'a>(Dir<'a, Disk, NullTimeProvider, LossyOemCpConverter>);

unsafe impl Sync for FatFileSystem {}

unsafe impl Send for FatFileSystem {}

unsafe impl<'a> Send for FileWrapper<'a> {}

unsafe impl<'a> Sync for FileWrapper<'a> {}

unsafe impl<'a> Send for DirWrapper<'a> {}

unsafe impl<'a> Sync for DirWrapper<'a> {}

impl FatFileSystem {
    /// 初始化一个FatFileSystem
    pub fn new(disk: Disk) -> Self {
        let inner = fatfs::FileSystem::new(disk, fatfs::FsOptions::new())
            .expect("failed to initialize FAT filesystem");
        Self {
            inner,
            root_dir: UnsafeCell::new(None),
        }
    }

    /// 设置root_dir,必须在其他操作前调用
    pub fn init(&'static self) {
        // must be called before later operations
        unsafe { *self.root_dir.get() = Some(Self::new_dir(self.inner.root_dir())) }
    }

    /// 从fatfs库的File创建一个FileWrapper
    fn new_file(file: File<'_, Disk, NullTimeProvider, LossyOemCpConverter>) -> Arc<FileWrapper> {
        Arc::new(FileWrapper(Mutex::new(file)))
    }

    /// 从fatfs库的Dir创建一个DirWrapper
    fn new_dir(dir: Dir<'_, Disk, NullTimeProvider, LossyOemCpConverter>) -> Arc<DirWrapper> {
        Arc::new(DirWrapper(dir))
    }
}

/// 实现VfsNodeOps trait以提供文件相关操作的抽象接口
impl VfsNodeOps for FileWrapper<'static> {
    axfs_vfs::impl_vfs_non_dir_default! {}

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        let size = self.0.lock().seek(SeekFrom::End(0)).map_err(as_vfs_err)?;
        let blocks = (size + BLOCK_SIZE as u64 - 1) / BLOCK_SIZE as u64;
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
        file.seek(SeekFrom::Start(size)).map_err(as_vfs_err)?; // TODO: more efficient
        file.truncate().map_err(as_vfs_err)
    }
}

/// 实现VfsNodeOps trait以提供目录相关操作的抽象接口
impl VfsNodeOps for DirWrapper<'static> {
    axfs_vfs::impl_vfs_dir_default! {}  //实现默认的目录操作
    // 包括 read_at() write_at() fsync() truncate(), 这些操作在目录中是无效的,所以直接返回错误

    /// 获取目录属性
    /// FAT文件系统不支持权限,所以设置为755
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        // FAT fs doesn't support permissions, we just set everything to 755
        Ok(VfsNodeAttr::new(
            VfsNodePerm::from_bits_truncate(0o755),
            VfsNodeType::Dir,
            BLOCK_SIZE as u64,
            1,
        ))
    }

    /// 获取父目录
    fn parent(&self) -> Option<VfsNodeRef> {
        self.0
            .open_dir("..")
            .map_or(None, |dir| Some(FatFileSystem::new_dir(dir)))
    }

    /// 查找目录中的文件或子目录
    fn lookup(self: Arc<Self>, path: &str) -> VfsResult<VfsNodeRef> {
        debug!("lookup at fatfs: {}", path);
        let path = path.trim_matches('/');
        if path.is_empty() || path == "." {
            return Ok(self.clone());
        }
        if let Some(rest) = path.strip_prefix("./") {
            return self.lookup(rest);
        }

        // TODO: use `fatfs::Dir::find_entry`, but it's not public.
        if let Ok(file) = self.0.open_file(path) {
            Ok(FatFileSystem::new_file(file))
        } else if let Ok(dir) = self.0.open_dir(path) {
            Ok(FatFileSystem::new_dir(dir))
        } else {
            Err(VfsError::NotFound)
        }
    }

    /// 在目录中创建文件或子目录
    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult {
        debug!("create {:?} at fatfs: {}", ty, path);
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

    /// 从目录中删除文件或子目录
    fn remove(&self, path: &str) -> VfsResult {
        debug!("remove at fatfs: {}", path);
        let path = path.trim_matches('/');
        assert!(!path.is_empty()); // already check at `root.rs`
        if let Some(rest) = path.strip_prefix("./") {
            return self.remove(rest);
        }
        self.0.remove(path).map_err(as_vfs_err)
    }

    /// 读取目录项
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
}

impl VfsOps for FatFileSystem {
    /// 根目录
    fn root_dir(&self) -> VfsNodeRef {
        let root_dir = unsafe { (*self.root_dir.get()).as_ref().unwrap() };
        root_dir.clone()
    }
}


impl fatfs::IoBase for Disk {
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

/// 将fatfs库的Error映射为VfsError
const fn as_vfs_err(err: fatfs::Error<()>) -> VfsError {
    use fatfs::Error::*;
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

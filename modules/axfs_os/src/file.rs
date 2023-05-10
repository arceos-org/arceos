use super::file_io::FileIO;
use crate::flags::OpenFlags;
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::string::String;
use axerrno::{AxError, AxResult};
use axfs::api::{DirEntry, File, OpenOptions};
use axfs::fops::Directory;
use axio::{Read, Seek, SeekFrom, Write};
use axsync::Mutex;
/// 文件描述符
pub struct FileDesc {
    /// 文件
    file: Arc<Mutex<File>>,
    /// 文件打开的标志位, 由低到高: read, write, append, truncate, create, create_new, 0, 0
    flags: OpenFlags,
}

/// 为FileDesc实现FileIO trait
impl FileIO for FileDesc {
    fn readable(&self) -> bool {
        self.flags.readable()
    }

    fn writable(&self) -> bool {
        self.flags.writable()
    }

    fn read(&self, buf: &mut [u8]) -> AxResult<usize> {
        self.file.lock().read(buf)
    }

    fn write(&self, buf: &[u8]) -> AxResult<usize> {
        self.file.lock().write(buf)
    }

    fn seek(&self, offset: usize) -> AxResult<u64> {
        self.file.lock().seek(SeekFrom::Start(offset as u64))
    }
    // fn flush(&self) {
    //     unsafe {
    //         (*self.file.get()).flush()
    //     }
    // }
    // fn set_len(&self, len: usize) {
    //     unsafe {
    //         (*self.file.get()).set_len(len)
    //     }
    // }
    // fn sync_all(&self) {
    //     unsafe {
    //         (*self.file.get()).sync_all()
    //     }
    // }
    // fn sync_data(&self) {
    //     unsafe {
    //         (*self.file.get()).sync_data()
    //     }
    // }
}

/// 文件描述符的实现
impl FileDesc {
    /// 创建一个新的文件描述符
    pub fn new(file: Arc<Mutex<File>>, flags: u8) -> Self {
        Self {
            file,
            flags: OpenFlags::from_bits(flags as u32).unwrap(),
        }
    }
    // /// 从文件描述符对应的文件进行读取，指定了开始的偏移量与读取的长度
    // pub fn read_file(&self, offset: usize, len: usize) -> AxResult<Vec<u8>, AxError> {
    //     let mut buf = [0u8; len];
    //     self.file
    //         .lock()
    //         .seek(SeekFrom::Start(offset as u64))
    //         .unwrap();
    //     self.file.lock().read_exact(&mut buf)?;
    //     Ok(buf.to_vec())
    // }
}

/// 新建一个文件描述符
pub fn new_fd(path: &str, flags: u8) -> AxResult<FileDesc> {
    let file = File::options()
        .read(flags & 0b0000_0001 != 0)
        .write(flags & 0b0000_0010 != 0)
        .append(flags & 0b0000_0100 != 0)
        .truncate(flags & 0b0000_1000 != 0)
        .create(flags & 0b0001_0000 != 0)
        .create_new(flags & 0b0010_0000 != 0)
        .open(path)?;
    // let file_size = file.metadata()?.len();
    let fd = FileDesc::new(Arc::new(Mutex::new(file)), flags);
    Ok(fd)
}

/// 工作目录描述符
pub struct CurWorkDirDesc {
    /// 工作目录
    //TODO: work_dir: Arc<Mutex<Directory>>, 支持更复杂的操作
    work_dir: String,
}
/// 工作目录描述符的实现
impl CurWorkDirDesc{
    pub fn new(work_dir: String) -> Self {
        Self { work_dir }
    }
    /// 获取工作目录
    pub fn get_path(&self) -> String {
        self.work_dir.clone()
    }
}

/// 为WorkDirDesc实现FileIO trait
impl FileIO for CurWorkDirDesc {
    fn readable(&self) -> bool {
        false
    }

    fn writable(&self) -> bool {
        false
    }

    fn read(&self, buf: &mut [u8]) -> AxResult<usize> {
        Err(AxError::IsADirectory)
    }

    fn write(&self, buf: &[u8]) -> AxResult<usize> {
        Err(AxError::IsADirectory)
    }
}
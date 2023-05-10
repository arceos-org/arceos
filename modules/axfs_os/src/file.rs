use super::file_io::FileIO;
use crate::flags::OpenFlags;
use alloc::sync::Arc;
use axerrno::AxResult;
use axfs::api::File;
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
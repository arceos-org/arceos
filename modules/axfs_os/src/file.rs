use alloc::string::{String, ToString};
use super::file_io::FileIO;
use crate::flags::OpenFlags;
use alloc::sync::Arc;
use log::debug;
use axerrno::AxResult;
use axfs::api::File;
use axio::{Read, Seek, SeekFrom, Write};
use axsync::Mutex;
use crate::FilePath;
use crate::link::get_link_count;
use crate::types::{Kstat, normal_file_mode, StMode};

/// 文件描述符
pub struct FileDesc {
    /// 文件路径
    pub path: String,
    /// 文件
    pub file: Arc<Mutex<File>>,
    /// 文件打开的标志位
    pub flags: OpenFlags,
    /// 文件信息
    pub stat: Mutex<FileMetaData>,
}

/// 文件在os中运行时的可变信息
/// TODO: 暂时全部记为usize
pub struct FileMetaData {
    /// 最后一次访问时间
    pub atime: usize,
    /// 最后一次改变(modify)内容的时间
    pub mtime: usize,
    /// 最后一次改变(change)属性的时间
    pub ctime: usize,
    // /// 打开时的选项。
    // /// 主要用于判断 CLOEXEC，即 exec 时是否关闭。默认为 false。
    // pub flags: OpenFlags,
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
        debug!("Into function read, buf_len: {}", buf.len());
        self.file.lock().read(buf)
    }

    fn write(&self, buf: &[u8]) -> AxResult<usize> {
        self.file.lock().write(buf)
    }

    fn seek(&self, offset: usize) -> AxResult<u64> {
        self.file.lock().seek(SeekFrom::Start(offset as u64))
    }

    fn get_path(&self) -> String {
        self.path.clone()
    }

    fn get_type(&self) -> String {
        "FileDesc".to_string()
    }

    fn get_stat(&self) -> AxResult<Kstat> {
        let file = self.file.lock();
        let metadata = file.metadata()?;
        let raw_metadata = metadata.raw_metadata();
        let stat = self.stat.lock();

        let kstat = Kstat {
            st_dev: 1,
            st_ino: 1,
            st_mode: normal_file_mode(StMode::S_IFREG).bits(),
            st_nlink: get_link_count(&FilePath::new(self.path.as_str())) as u32,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            _pad0: 0,
            st_size: raw_metadata.size() as u64,
            st_blksize: 0,
            _pad1: 0,
            st_blocks: raw_metadata.blocks() as u64,
            st_atime_sec: stat.atime as isize,
            st_atime_nsec: 0,
            st_mtime_sec: stat.mtime as isize,
            st_mtime_nsec: 0,
            st_ctime_sec: stat.ctime as isize,
            st_ctime_nsec: 0,
        };
        Ok(kstat)
    }

    /// debug
    fn print_content(&self) {
        debug!("Into function print_content");
        let mut contents = String::new();
        self.file.lock().read_to_string(&mut contents).unwrap();
        debug!("{}", contents);
    }
}

/// 文件描述符的实现
impl FileDesc {
    /// 创建一个新的文件描述符
    pub fn new(path: &str, file: Arc<Mutex<File>>, flags: OpenFlags) -> Self {
        Self {
            path: path.to_string(),
            file,
            flags,
            stat: Mutex::new(FileMetaData {
                atime: 0,
                mtime: 0,
                ctime: 0,
            }),
        }
    }
}

/// 新建一个文件描述符
pub fn new_fd(path: String, flags: OpenFlags) -> AxResult<FileDesc> {
    debug!("Into function new_fd, path: {}", path);
    let mut file = File::options();
    file.read(flags.readable());
    file.write(flags.writable());
    file.create(flags.creatable());
    file.create_new(flags.new_creatable());
    let file = file.open(path.as_str())?;
    // let file_size = file.metadata()?.len();
    let fd = FileDesc::new(path.as_str(), Arc::new(Mutex::new(file)), flags);
    Ok(fd)
}


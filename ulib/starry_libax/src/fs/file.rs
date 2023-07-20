extern crate alloc;
use crate::syscall::flags::TimeSecs;

use super::link::get_link_count;
use super::types::{normal_file_mode, StMode};
use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use axerrno::AxResult;
use axfs::api::File;
use axfs::monolithic_fs::file_io::{FileExt, Kstat};
use axfs::monolithic_fs::flags::OpenFlags;
use axfs::monolithic_fs::FileIO;
use axfs::monolithic_fs::FileIOType;
use axhal::time::TimeValue;

use axio::{Read, Seek, SeekFrom, Write};
use axprocess::link::FilePath;
use axsync::Mutex;
use log::{debug, info};

use axfs::BLOCK_SIZE;

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
    pub atime: TimeSecs,
    /// 最后一次改变(modify)内容的时间
    pub mtime: TimeSecs,
    /// 最后一次改变(change)属性的时间
    pub ctime: TimeSecs,
    // /// 打开时的选项。
    // /// 主要用于判断 CLOEXEC，即 exec 时是否关闭。默认为 false。
    // pub flags: OpenFlags,
}

impl Read for FileDesc {
    fn read(&mut self, buf: &mut [u8]) -> AxResult<usize> {
        debug!("Into function read, buf_len: {}", buf.len());
        // 似乎当前的fat32文件系统不支持一次读取达到block size的内容
        let buf_len = buf.len();
        let mut offset = 0;
        while offset < buf_len {
            let read_len = self
                .file
                .lock()
                .read(&mut buf[offset..buf_len.min(offset + BLOCK_SIZE - 1)])?;
            if read_len == 0 {
                break;
            }
            offset += read_len;
        }
        Ok(offset)
    }
}

impl Write for FileDesc {
    fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        self.file.lock().write(buf)
    }

    fn flush(&mut self) -> AxResult {
        self.file.lock().flush()
    }
}

impl Seek for FileDesc {
    fn seek(&mut self, pos: SeekFrom) -> AxResult<u64> {
        self.file.lock().seek(pos)
    }
}

impl FileExt for FileDesc {
    fn readable(&self) -> bool {
        self.flags.readable()
    }
    fn writable(&self) -> bool {
        self.flags.writable()
    }
    fn executable(&self) -> bool {
        self.file.lock().executable()
    }
}
/// 为FileDesc实现FileIO trait
impl FileIO for FileDesc {
    fn get_type(&self) -> FileIOType {
        FileIOType::FileDesc
    }
    fn get_path(&self) -> String {
        self.path.clone()
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
            st_nlink: get_link_count(&(FilePath::new(self.path.as_str())?)) as u32,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            _pad0: 0,
            st_size: raw_metadata.size() as u64,
            st_blksize: 0,
            _pad1: 0,
            st_blocks: raw_metadata.blocks() as u64,
            st_atime_sec: stat.atime.tv_sec as isize,
            st_atime_nsec: stat.atime.tv_nsec as isize,
            st_mtime_sec: stat.mtime.tv_sec as isize,
            st_mtime_nsec: stat.mtime.tv_nsec as isize,
            st_ctime_sec: stat.ctime.tv_sec as isize,
            st_ctime_nsec: stat.ctime.tv_nsec as isize,
            _unused: [0; 2],
        };
        debug!("kstat: {:?}", kstat);
        Ok(kstat)
    }

    fn set_status(&mut self, flags: OpenFlags) -> bool {
        self.flags = flags;
        true
    }

    fn get_status(&self) -> OpenFlags {
        self.flags
    }

    fn set_close_on_exec(&mut self, is_set: bool) -> bool {
        if is_set {
            // 设置close_on_exec位置
            self.flags |= OpenFlags::CLOEXEC;
        } else {
            self.flags &= !OpenFlags::CLOEXEC;
        }
        true
    }

    fn ready_to_read(&mut self) -> bool {
        if !self.readable() {
            return false;
        }
        // 获取当前的位置
        let now_pos = self.seek(SeekFrom::Current(0)).unwrap();
        // 获取最后的位置
        let len = self.seek(SeekFrom::End(0)).unwrap();
        // 把文件指针复原，因为获取len的时候指向了尾部
        self.seek(SeekFrom::Start(now_pos)).unwrap();
        now_pos != len
    }

    fn ready_to_write(&mut self) -> bool {
        if !self.writable() {
            return false;
        }
        // 获取当前的位置
        let now_pos = self.seek(SeekFrom::Current(0)).unwrap();
        // 获取最后的位置
        let len = self.seek(SeekFrom::End(0)).unwrap();
        // 把文件指针复原，因为获取len的时候指向了尾部
        self.seek(SeekFrom::Start(now_pos)).unwrap();
        now_pos != len
    }
}

impl FileDesc {
    /// debug

    /// 创建一个新的文件描述符
    pub fn new(path: &str, file: Arc<Mutex<File>>, flags: OpenFlags) -> Self {
        Self {
            path: path.to_string(),
            file,
            flags,
            stat: Mutex::new(FileMetaData {
                atime: TimeSecs::default(),
                mtime: TimeSecs::default(),
                ctime: TimeSecs::default(),
            }),
        }
    }
}

/// 若使用多次new file打开同名文件，那么不同new file之间读写指针不共享，但是修改的内容是共享的
pub fn new_file(path: &str, flags: &OpenFlags) -> AxResult<File> {
    let mut file = File::options();
    file.read(flags.readable());
    file.write(flags.writable());
    file.create(flags.creatable());
    file.create_new(flags.new_creatable());
    file.open(path)
}

/// 新建一个文件描述符
pub fn new_fd(path: String, flags: OpenFlags) -> AxResult<FileDesc> {
    debug!("Into function new_fd, path: {}", path);
    let mut file = new_file(path.as_str(), &flags)?;
    // let file_size = file.metadata()?.len();
    let pos = file.seek(SeekFrom::Current(0))? as usize;
    let file_size = file.seek(SeekFrom::End(0))? as usize;
    info!("pos: {}, file_size: {}", pos, file_size);
    file.seek(SeekFrom::Start(pos as u64))?;
    let fd = FileDesc::new(path.as_str(), Arc::new(Mutex::new(file)), flags);
    Ok(fd)
}

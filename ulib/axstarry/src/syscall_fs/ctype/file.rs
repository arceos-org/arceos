extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use axerrno::AxResult;
use axfs::api::{File, FileIO, FileIOType, Kstat, OpenFlags, Read, Seek, SeekFrom, Write};

use axlog::debug;

use crate::{new_file, normal_file_mode, StMode, TimeSecs};
use axprocess::link::get_link_count;
use axsync::Mutex;

pub static INODE_NAME_MAP: Mutex<BTreeMap<String, u64>> = Mutex::new(BTreeMap::new());

/// 文件描述符
pub struct FileDesc {
    /// 文件路径
    pub path: String,
    /// 文件
    pub file: Arc<Mutex<File>>,
    /// 文件打开的标志位
    pub flags: Mutex<OpenFlags>,
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

/// 为FileDesc实现FileIO trait
impl FileIO for FileDesc {
    fn read(&self, buf: &mut [u8]) -> AxResult<usize> {
        self.file.lock().read(buf)
    }

    fn write(&self, buf: &[u8]) -> AxResult<usize> {
        // 如果seek时超出了文件原有大小，则在write的时候进行补零操作
        let mut file = self.file.lock();
        let old_offset = file.seek(SeekFrom::Current(0)).unwrap();
        let size = file.metadata().unwrap().size();
        if old_offset > size {
            file.seek(SeekFrom::Start(size)).unwrap();
            let temp_buf: Vec<u8> = vec![0u8; (old_offset - size) as usize];
            file.write(&temp_buf)?;
        }
        file.write(buf)
    }

    fn flush(&self) -> AxResult {
        self.file.lock().flush()
    }

    fn seek(&self, pos: SeekFrom) -> AxResult<u64> {
        self.file.lock().seek(pos)
    }

    fn readable(&self) -> bool {
        self.flags.lock().readable()
    }
    fn writable(&self) -> bool {
        self.flags.lock().writable()
    }
    fn executable(&self) -> bool {
        self.file.lock().executable()
    }

    fn get_type(&self) -> FileIOType {
        FileIOType::FileDesc
    }
    fn get_path(&self) -> String {
        self.path.clone()
    }

    fn truncate(&self, len: usize) -> AxResult<()> {
        self.file.lock().truncate(len)
    }

    fn get_stat(&self) -> AxResult<Kstat> {
        let file = self.file.lock();
        let attr = file.get_attr()?;
        let stat = self.stat.lock();
        let inode_map = INODE_NAME_MAP.lock();
        let inode_number = if let Some(inode_number) = inode_map.get(&self.path) {
            *inode_number
        } else {
            // return Err(axerrno::AxError::NotFound);
            // Now the file exists but it wasn't opened
            drop(inode_map);
            new_inode(self.path.clone())?;
            let inode_map = INODE_NAME_MAP.lock();
            assert!(inode_map.contains_key(&self.path));
            let number = *(inode_map.get(&self.path).unwrap());
            drop(inode_map);
            number
        };
        let kstat = Kstat {
            st_dev: 1,
            st_ino: inode_number,
            st_mode: normal_file_mode(StMode::S_IFREG).bits() | 0o644,
            st_nlink: get_link_count(&(self.path.as_str().to_string())) as _,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            _pad0: 0,
            st_size: attr.size(),
            st_blksize: axfs::BLOCK_SIZE as u32,
            _pad1: 0,
            st_blocks: attr.blocks(),
            st_atime_sec: stat.atime.tv_sec as isize,
            st_atime_nsec: stat.atime.tv_nsec as isize,
            st_mtime_sec: stat.mtime.tv_sec as isize,
            st_mtime_nsec: stat.mtime.tv_nsec as isize,
            st_ctime_sec: stat.ctime.tv_sec as isize,
            st_ctime_nsec: stat.ctime.tv_nsec as isize,
        };
        Ok(kstat)
    }

    fn set_status(&self, flags: OpenFlags) -> bool {
        *self.flags.lock() = flags;
        true
    }

    fn get_status(&self) -> OpenFlags {
        *self.flags.lock()
    }

    fn set_close_on_exec(&self, is_set: bool) -> bool {
        if is_set {
            // 设置close_on_exec位置
            *self.flags.lock() |= OpenFlags::CLOEXEC;
        } else {
            *self.flags.lock() &= !OpenFlags::CLOEXEC;
        }
        true
    }

    fn ready_to_read(&self) -> bool {
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

    fn ready_to_write(&self) -> bool {
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
            flags: Mutex::new(flags),
            stat: Mutex::new(FileMetaData {
                atime: TimeSecs::default(),
                mtime: TimeSecs::default(),
                ctime: TimeSecs::default(),
            }),
        }
    }
}

/// 新建一个文件描述符
pub fn new_fd(path: String, flags: OpenFlags) -> AxResult<FileDesc> {
    debug!("Into function new_fd, path: {}", path);
    let file = new_file(path.as_str(), &flags)?;
    // let file_size = file.metadata()?.len();

    let fd = FileDesc::new(path.as_str(), Arc::new(Mutex::new(file)), flags);
    Ok(fd)
}

/// 当新建一个文件或者目录节点时，需要为其分配一个新的inode号
/// 由于我们不涉及删除文件，因此我们可以简单地使用一个全局增的计数器来分配inode号
pub fn new_inode(path: String) -> AxResult<()> {
    let mut inode_name_map = INODE_NAME_MAP.lock();
    if inode_name_map.contains_key(&path) {
        return Ok(());
    }
    let inode_number = inode_name_map.len() as u64 + 1;
    inode_name_map.insert(path, inode_number);
    Ok(())
}

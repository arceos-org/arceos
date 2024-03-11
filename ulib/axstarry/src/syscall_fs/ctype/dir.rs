use crate::{normal_file_mode, StMode};
extern crate alloc;
use alloc::string::{String, ToString};
use axerrno::{AxError, AxResult};
use axfs::api::{self, FileIO, FileIOType, Kstat, OpenFlags, SeekFrom};
use axlog::debug;

/// 目录描述符
pub struct DirDesc {
    /// 目录
    pub dir_path: String,
}

/// 目录描述符的实现
impl DirDesc {
    /// 创建一个新的目录描述符
    pub fn new(path: String) -> Self {
        Self { dir_path: path }
    }
}

/// 为DirDesc实现FileIO trait
impl FileIO for DirDesc {
    fn read(&self, _: &mut [u8]) -> AxResult<usize> {
        Err(AxError::IsADirectory)
    }
    fn write(&self, _: &[u8]) -> AxResult<usize> {
        Err(AxError::IsADirectory)
    }
    fn flush(&self) -> AxResult {
        Err(AxError::IsADirectory)
    }
    fn seek(&self, _: SeekFrom) -> AxResult<u64> {
        Err(AxError::IsADirectory)
    }
    fn get_type(&self) -> FileIOType {
        FileIOType::DirDesc
    }
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        false
    }
    fn executable(&self) -> bool {
        false
    }
    fn get_path(&self) -> String {
        self.dir_path.to_string().clone()
    }

    fn get_stat(&self) -> AxResult<Kstat> {
        let kstat = Kstat {
            st_dev: 1,
            st_ino: 0,
            st_mode: normal_file_mode(StMode::S_IFDIR).bits(),
            st_nlink: 1,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            _pad0: 0,
            st_size: 0,
            st_blksize: 0,
            _pad1: 0,
            st_blocks: 0,
            st_atime_sec: 0,
            st_atime_nsec: 0,
            st_mtime_sec: 0,
            st_mtime_nsec: 0,
            st_ctime_sec: 0,
            st_ctime_nsec: 0,
        };
        Ok(kstat)
    }
}

pub fn new_dir(dir_path: String, _flags: OpenFlags) -> AxResult<DirDesc> {
    debug!("Into function new_dir, dir_path: {}", dir_path);
    if !api::path_exists(dir_path.as_str()) {
        // api::create_dir_all(dir_path.as_str())?;
        api::create_dir(dir_path.as_str())?;
    }
    Ok(DirDesc::new(dir_path))
}

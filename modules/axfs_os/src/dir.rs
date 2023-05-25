use alloc::string::{String, ToString};
use log::debug;
use axerrno::{AxError, AxResult};
use axfs::api;
use crate::file_io::FileIO;
use crate::flags::OpenFlags;

/// 目录描述符
pub struct DirDesc {
    /// 目录
    pub dir_path: String,
}

/// 目录描述符的实现
impl DirDesc {
    /// 创建一个新的目录描述符
    pub fn new(path: String) -> Self {
        Self {
            dir_path: path,
        }
    }
}

/// 为DirDesc实现FileIO trait
impl FileIO for DirDesc {
    fn readable(&self) -> bool {
        false
    }

    fn writable(&self) -> bool {
        false
    }

    fn read(&self, _buf: &mut [u8]) -> AxResult<usize> {
        Err(AxError::IsADirectory)
    }

    fn write(&self, _buf: &[u8]) -> AxResult<usize> {
        Err(AxError::IsADirectory)
    }

    fn get_path(&self) -> String {
        self.dir_path.to_string().clone()
    }

    fn get_type(&self) -> String {
        "DirDesc".to_string()
    }
}

pub fn new_dir(dir_path: String, _flags: OpenFlags) -> AxResult<DirDesc> {
    debug!("Into function new_dir, dir_path: {}", dir_path);
    if let Err(e) = api::read_dir(dir_path.as_str()) {
        return Err(e);
    }
    Ok(DirDesc::new(dir_path))
}

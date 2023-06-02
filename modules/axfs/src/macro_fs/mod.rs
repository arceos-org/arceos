pub mod file_io;
pub use crate::api;
use crate::api::OpenOptions;
use axerrno::AxResult;
use axio::{Read, Seek, SeekFrom};
pub use file_io::{FileIO, FileIOType};
use log::info;
pub mod flags;

extern crate alloc;

use alloc::vec::Vec;

/// 读取path文件的内容，但不新建文件描述符
/// 用于内核读取代码文件初始化
pub fn read_file(path: &str) -> AxResult<Vec<u8>> {
    if let Ok(mut file) = OpenOptions::new().read(true).open(path) {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).expect("failed to read file");
        return Ok(buf);
    }
    info!("failed to open file {}", path);
    Err(axerrno::AxError::NotFound)
}

/// 读取文件, 从指定位置开始读取完整内容
pub fn read_file_with_offset(path: &str, offset: isize) -> AxResult<Vec<u8>> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(path)
        .expect("failed to open file");
    file.seek(SeekFrom::Start(offset as u64))
        .expect("failed to seek file");
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).expect("failed to read file");
    Ok(buf)
}

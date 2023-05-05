#![cfg_attr(not(test), no_std)]

pub mod file;
pub mod file_io;
pub mod stdio;
extern crate alloc;
use alloc::vec::Vec;
pub mod flags;
use axerrno::AxResult;
use axfs::api::OpenOptions;
use axio::{Read, Seek, SeekFrom};
pub use file::new_fd;
pub use stdio::{Stderr, Stdin, Stdout};

/// 读取path文件的内容，但不新建文件描述符
/// 用于内核读取代码文件初始化
pub fn read_file(path: &str) -> AxResult<Vec<u8>> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(path)
        .expect("failed to open file");
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).expect("failed to read file");
    Ok(buf)
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

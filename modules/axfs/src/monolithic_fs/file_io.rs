/// 定义与文件I/O操作相关的trait泛型
use axerrno::{AxError, AxResult};
use axio::{Read, Seek, SeekFrom, Write};
use core::any::Any;
use log::debug;
extern crate alloc;
use alloc::string::String;
/// 文件系统信息
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Kstat {
    /// 设备
    pub st_dev: u64,
    /// inode 编号
    pub st_ino: u64,
    /// 文件类型
    pub st_mode: u32,
    /// 硬链接数
    pub st_nlink: u32,
    /// 用户id
    pub st_uid: u32,
    /// 用户组id
    pub st_gid: u32,
    /// 设备号
    pub st_rdev: u64,
    pub _pad0: u32,
    /// 文件大小
    pub st_size: u64,
    /// 块大小
    pub st_blksize: u32,
    pub _pad1: u32,
    /// 块个数
    pub st_blocks: u64,
    /// 最后一次访问时间(秒)
    pub st_atime_sec: isize,
    /// 最后一次访问时间(纳秒)
    pub st_atime_nsec: isize,
    /// 最后一次修改时间(秒)
    pub st_mtime_sec: isize,
    /// 最后一次修改时间(纳秒)
    pub st_mtime_nsec: isize,
    /// 最后一次改变状态时间(秒)
    pub st_ctime_sec: isize,
    /// 最后一次改变状态时间(纳秒)
    pub st_ctime_nsec: isize,
    pub _unused: [u32; 2],
}

/// 文件类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileIOType {
    /// 文件
    FileDesc,
    /// 目录
    DirDesc,
    /// 标准输入输出错误流
    Stdin,
    Stdout,
    Stderr,
    /// 管道
    Pipe,
    /// 链接
    Link,
    /// Socket
    Socket,
    /// 其他
    Other,
}

/// File I/O trait. 文件I/O操作，用于设置文件描述符
pub trait FileIO: FileExt {
    /// 获取类型
    fn get_type(&self) -> FileIOType;

    /// 获取路径
    fn get_path(&self) -> String {
        debug!("Function get_path not implemented");
        String::from("Function get_path not implemented")
    }
    /// 获取文件信息
    fn get_stat(&self) -> AxResult<Kstat> {
        Err(AxError::Unsupported) // 如果没有实现get_stat, 则返回Unsupported
    }

    /// debug
    fn print_content(&self) {
        debug!("Function print_content not implemented");
    }
}

/// `FileExt` 需要满足 `AsAny` 的要求，即可以转化为 `Any` 类型，从而能够进行向下类型转换。
pub trait AsAny {
    /// 把当前对象转化为 `Any` 类型，供后续 downcast 使用
    fn as_any(&self) -> &dyn Any;
    // 供 downcast_mut 使用
    // fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    // fn as_any_mut(&mut self) -> &mut dyn Any {
    //     self
    // }
}

/// 用于给虚存空间进行懒分配
#[cfg(feature = "monolithic")]
pub trait FileExt: Read + Write + Seek + AsAny + Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn executable(&self) -> bool;
    /// Read from position without changing cursor.
    fn read_from_seek(&mut self, pos: SeekFrom, buf: &mut [u8]) -> AxResult<usize> {
        // get old position
        let old_pos = self
            .seek(SeekFrom::Current(0))
            .expect("Error get current pos in file");
        info!("    'read_from_seek' old_pos: {}", old_pos);

        // seek to read position
        let _ = self.seek(pos).unwrap();

        // read
        let read_len = self.read_full(buf);

        // seek back to old_pos
        let new_pos = self.seek(SeekFrom::Start(old_pos)).unwrap();

        assert_eq!(old_pos, new_pos);

        read_len
    }

    /// Write to position without changing cursor.
    fn write_to_seek(&mut self, pos: SeekFrom, buf: &[u8]) -> AxResult<usize> {
        // get old position
        let old_pos = self
            .seek(SeekFrom::Current(0))
            .expect("Error get current pos in file");

        // seek to write position
        let _ = self.seek(pos).unwrap();

        let write_len = self.write(buf);

        // seek back to old_pos
        let _ = self.seek(SeekFrom::Start(old_pos)).unwrap();

        write_len
    }
}

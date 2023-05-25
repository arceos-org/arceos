use alloc::string::String;
use core::any::Any;
use log::debug;
use axerrno::{AxError, AxResult};
use crate::types::Kstat;

/// File I/O trait. 文件I/O操作
pub trait FileIO: Send + Sync + AsAny {
    /// 文件是否可读
    fn readable(&self) -> bool;
    /// 文件是否可写
    fn writable(&self) -> bool;
    /// 读取文件数据到缓冲区, 返回读取的字节数
    fn read(&self, buf: &mut [u8]) -> AxResult<usize>;
    /// 将缓冲区数据写入文件, 返回写入的字节数
    fn write(&self, buf: &[u8]) -> AxResult<usize>;
    /// 移动文件指针, 返回新的文件指针位置
    fn seek(&self, _pos: usize) -> AxResult<u64> {
        Err(AxError::Unsupported) // 如果没有实现seek, 则返回Unsupported
    }
    /// 刷新文件缓冲区
    fn flush(&self) -> AxResult<()> {
        Err(AxError::Unsupported) // 如果没有实现flush, 则返回Unsupported
    }
    /// 获取路径
    fn get_path(&self) -> String {
        debug!("Function get_path not implemented");
        String::from("Function get_path not implemented")
    }
    /// 获取类型
    fn get_type(&self) -> String;
    /// 获取文件信息
    fn get_stat(&self) -> AxResult<Kstat> {
        Err(AxError::Unsupported) // 如果没有实现get_stat, 则返回Unsupported
    }

    /// debug
    fn print_content(&self) {
        debug!("Function print_content not implemented");
    }
}

/// `FileIO` 需要满足 `AsAny` 的要求，即可以转化为 `Any` 类型，从而能够进行向下类型转换。
pub trait AsAny {
    /// 把当前对象转化为 `Any` 类型，供后续 downcast 使用
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any { self }
}

use axerrno::{AxError, AxResult};

/// File I/O trait. 文件I/O操作
pub trait FileIO: Send + Sync {
    /// 文件是否可读
    fn readable(&self) -> bool;
    /// 文件是否可写
    fn writable(&self) -> bool;
    /// 读取文件数据到缓冲区, 返回读取的字节数
    fn read(&self, buf: &mut [u8]) -> AxResult<usize>;
    /// 将缓冲区数据写入文件, 返回写入的字节数
    fn write(&self, buf: &[u8]) -> AxResult<usize>;

    /// 移动文件指针, 返回新的文件指针位置
    fn seek(&self, _pos: usize) -> AxResult<u64>;
    /// 刷新文件缓冲区
    fn flush(&self) -> AxResult<()> {
        Err(AxError::Unsupported) // 如果没有实现flush, 则返回Unsupported
    }
}

//! todo 重构fd_table, fd_allocator

use alloc::sync::Arc;
use axfs::monolithic_fs::FileIO;
use spinlock::SpinNoIrq;
extern crate alloc;
use alloc::vec::Vec;
pub struct FdManager {
    /// 保存文件描述符的数组
    pub fd_table: Vec<Option<Arc<SpinNoIrq<dyn FileIO>>>>,
    /// 保存文件描述符的数组的最大长度
    pub limit: usize,
    /// 创建文件时的mode需要屏蔽这些位
    /// 如 sys_open 时权限为 0o666 ，而调用者的 umask 为 0o022，则实际权限为 0o644
    umask: i32,
}

impl FdManager {
    pub fn new(fd_table: Vec<Option<Arc<SpinNoIrq<dyn FileIO>>>>, limit: usize) -> Self {
        Self {
            fd_table,
            limit,
            umask: 0o022,
        }
    }

    pub fn get_mask(&self) -> i32 {
        self.umask
    }
    /// 设置新的 mask，返回旧的 mask
    pub fn set_mask(&mut self, new_mask: i32) -> i32 {
        let old_mask = self.umask;
        self.umask = new_mask;
        old_mask
    }
}

/// 实现文件描述符等相关文件操作
/// #![cfg_attr(not(test), no_std)]
#[allow(unused)]
pub mod file;

extern crate alloc;
#[allow(unused)]
pub mod dir;
#[allow(unused)]
pub mod link;
#[allow(unused)]
pub mod mount;
#[allow(unused)]
pub mod pipe;
pub use axfs::api;
use bitflags::bitflags;
pub use dir::{new_dir, DirDesc};
pub use file::{new_fd, FileDesc, FileMetaData};
pub use pipe::Pipe;
bitflags! {
    /// 指定 st_mode 的选项
    struct StMode: u32 {
        /// 是普通文件
        const S_IFREG = 1 << 15;
        /// 是目录
        const S_IFDIR = 1 << 14;
        /// 是字符设备
        const S_IFCHR = 1 << 13;
        /// 是否设置 uid/gid/sticky
        //const S_ISUID = 1 << 14;
        //const S_ISGID = 1 << 13;
        //const S_ISVTX = 1 << 12;
        /// 所有者权限
        const S_IXUSR = 1 << 10;
        const S_IWUSR = 1 << 9;
        const S_IRUSR = 1 << 8;
        /// 用户组权限
        const S_IXGRP = 1 << 6;
        const S_IWGRP = 1 << 5;
        const S_IRGRP = 1 << 4;
        /// 其他用户权限
        const S_IXOTH = 1 << 2;
        const S_IWOTH = 1 << 1;
        const S_IROTH = 1 << 0;
        /// 报告已执行结束的用户进程的状态
        const WIMTRACED = 1 << 1;
        /// 报告还未结束的用户进程的状态
        const WCONTINUED = 1 << 3;
    }
}
/// 文件类型，输入 IFCHR / IFDIR / IFREG 等具体类型，
/// 输出这些类型加上普遍的文件属性后得到的 mode 参数
fn normal_file_mode(file_type: StMode) -> StMode {
    file_type | StMode::S_IWUSR | StMode::S_IWUSR | StMode::S_IWGRP | StMode::S_IRGRP
}

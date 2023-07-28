use bitflags::*;

bitflags! {
    /// 指定文件打开时的权限
    #[derive(Clone, Copy, Default)]
    pub struct OpenFlags: u32 {
        /// 只读
        const RDONLY = 0;
        /// 只能写入
        const WRONLY = 1 << 0;
        /// 读写
        const RDWR = 1 << 1;
        /// 如文件不存在，可创建它
        const CREATE = 1 << 6;
        /// 确认一定是创建文件。如文件已存在，返回 EEXIST。
        const EXCLUSIVE = 1 << 7;
        /// 使打开的文件不会成为该进程的控制终端。目前没有终端设置，不处理
        const NOCTTY = 1 << 8;
        /// 同上，在不同的库中可能会用到这个或者上一个
        const EXCL = 1 << 9;
        /// 非阻塞读写?(虽然不知道为什么但 date.lua 也要)
        /// 在 socket 中使用得较多
        const NON_BLOCK = 1 << 11;
        /// 要求把 CR-LF 都换成 LF
        const TEXT = 1 << 14;
        /// 和上面不同，要求输入输出都不进行这个翻译
        const BINARY = 1 << 15;
        /// 对这个文件的输出需符合 IO 同步一致性。可以理解为随时 fsync
        const DSYNC = 1 << 16;
        /// 如果是符号链接，不跟随符号链接去寻找文件，而是针对连接本身
        const NOFOLLOW = 1 << 17;
        /// 在 exec 时需关闭
        const CLOEXEC = 1 << 19;
        /// 是否是目录
        const DIR = 1 << 21;
    }
}

impl OpenFlags {
    /// 获得文件的读/写权限
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
    /// 获取读权限
    pub fn readable(&self) -> bool {
        !self.contains(Self::WRONLY)
    }
    /// 获取写权限
    pub fn writable(&self) -> bool {
        self.contains(Self::WRONLY) || self.contains(Self::RDWR)
    }

    /// 获取创建权限
    pub fn creatable(&self) -> bool {
        self.contains(Self::CREATE)
    }
    /// 获取创建新文件权限
    /// 与上面的区别是，如果文件已存在，返回 EEXIST
    pub fn new_creatable(&self) -> bool {
        self.contains(Self::EXCLUSIVE)
    }

    /// 获取是否是目录
    pub fn is_dir(&self) -> bool {
        self.contains(Self::DIR)
    }

    /// 获取是否需要在 `exec()` 时关闭
    pub fn is_close_on_exec(&self) -> bool {
        self.contains(Self::CLOEXEC)
    }
}

impl From<usize> for OpenFlags {
    fn from(val: usize) -> Self {
        Self::from_bits_truncate(val as u32)
    }
}

bitflags! {
    /// 指定文件打开时的权限
    #[derive(Clone, Copy)]
    pub struct AccessMode: u16 {
        /// 用户读权限
        const S_IRUSR = 1 << 8;
        /// 用户写权限
        const S_IWUSR = 1 << 7;
        /// 用户执行权限
        const S_IXUSR = 1 << 6;
        /// 用户组读权限
        const S_IRGRP = 1 << 5;
        /// 用户组写权限
        const S_IWGRP = 1 << 4;
        /// 用户组执行权限
        const S_IXGRP = 1 << 3;
        /// 其他用户读权限
        const S_IROTH = 1 << 2;
        /// 其他用户写权限
        const S_IWOTH = 1 << 1;
        /// 其他用户执行权限
        const S_IXOTH = 1 << 0;
    }
}

impl From<usize> for AccessMode {
    fn from(val: usize) -> Self {
        Self::from_bits_truncate(val as u16)
    }
}

/// IOCTL系统调用支持
pub const TCGETS: usize = 0x5401;
pub const TIOCGPGRP: usize = 0x540F;
pub const TIOCSPGRP: usize = 0x5410;
pub const TIOCGWINSZ: usize = 0x5413;
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ConsoleWinSize {
    pub ws_row: u16,
    pub ws_col: u16,
    pub ws_xpixel: u16,
    pub ws_ypixel: u16,
}

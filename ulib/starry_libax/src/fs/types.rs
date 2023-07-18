extern crate alloc;
use alloc::string::String;
use axfs::api::canonicalize;
use bitflags::bitflags;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct FilePath(String);

impl FilePath {
    /// 创建一个 FilePath, 传入的 path 会被 canonicalize, 故可以是相对路径
    pub fn new(path: &str) -> Self {
        let mut new_path = String::from(canonicalize(path).unwrap().trim());
        // canonicalize中没有处理末尾的空格、换行符等
        if path.ends_with("/") && !new_path.ends_with("/") {
            // 如果原始路径以 '/' 结尾，那么canonicalize后的路径也应该以 '/' 结尾
            new_path.push('/');
        }
        // assert!(!path.ends_with("/"), "path should not end with '/', link only support file");      // 链接只支持文件
        Self(new_path)
    }
    /// 获取路径
    pub fn path(&self) -> &str {
        &self.0
    }
    /// 获取所属目录
    #[allow(unused)]
    pub fn dir(&self) -> &str {
        if self.is_root() {
            return "/";
        }
        let mut pos = self.0.rfind("/").unwrap();
        if pos == self.0.len() - 1 {
            pos = self.0[..pos].rfind("/").unwrap(); // 如果是以 '/' 结尾，那么再往前找一次
        }
        &self.0[..=pos]
    }
    /// 获取文件/目录名
    #[allow(unused)]
    pub fn file(&self) -> &str {
        if self.is_root() {
            return "/";
        }
        let mut pos = self.0.rfind("/").unwrap();
        if pos == self.0.len() - 1 {
            pos = self.0[..pos].rfind("/").unwrap(); // 如果是以 '/' 结尾，那么再往前找一次
        }
        &self.0[pos + 1..]
    }
    /// 返回是否是根目录
    #[allow(unused)]
    pub fn is_root(&self) -> bool {
        self.0 == "/"
    }
    /// 返回是否是目录
    /// 只能判断路径本身的格式，并不会考虑实际的文件信息
    pub fn is_dir(&self) -> bool {
        self.0.ends_with("/")
    }
    /// 返回是否是文件
    /// 只能判断路径本身的格式，并不会考虑实际的文件信息
    pub fn is_file(&self) -> bool {
        !self.0.ends_with("/")
    }
    /// 判断是否相同
    pub fn equal_to(&self, other: &Self) -> bool {
        self.0 == other.0
    }
    // /// 判断是否实际存在于文件系统(而不是只有链接)
    // pub fn exists(&self) -> bool {
    //     let path = self.0.clone();
    //     path_exists(path.as_str())
    // }
    /// 判断是否start_with
    pub fn start_with(&self, other: &Self) -> bool {
        self.0.starts_with(other.0.as_str())
    }
    /// 判断是否end_with
    #[allow(unused)]
    pub fn end_with(&self, other: &Self) -> bool {
        self.0.ends_with(other.0.as_str())
    }
}

// impl Clone for FilePath {
//     fn clone(&self) -> Self {
//         Self(self.0.clone())
//     }
// }

/// 目录项
#[repr(C)]
pub struct DirEnt {
    /// 索引结点号
    pub d_ino: u64,
    /// 到下一个dirent的偏移
    pub d_off: i64,
    /// 当前dirent的长度
    pub d_reclen: u16,
    /// 文件类型
    pub d_type: u8,
    /// 文件名
    pub d_name: [u8; 0],
}

#[allow(unused)]
pub enum DirEntType {
    /// 未知类型文件
    UNKNOWN = 0,
    /// 先进先出的文件/队列
    FIFO = 1,
    /// 字符设备
    CHR = 2,
    /// 目录
    DIR = 4,
    /// 块设备
    BLK = 6,
    /// 常规文件
    REG = 8,
    /// 符号链接
    LNK = 10,
    /// socket
    SOCK = 12,
    WHT = 14,
}

impl DirEnt {
    /// 定长部分大小
    pub fn fixed_size() -> usize {
        8 + 8 + 2 + 1
    }
    /// 设置定长部分
    pub fn set_fixed_part(&mut self, ino: u64, off: i64, reclen: usize, type_: DirEntType) {
        self.d_ino = ino;
        self.d_off = off;
        self.d_reclen = reclen as u16;
        self.d_type = type_ as u8;
    }
}

bitflags! {
    /// 指定 st_mode 的选项
    pub struct StMode: u32 {
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
pub fn normal_file_mode(file_type: StMode) -> StMode {
    file_type | StMode::S_IWUSR | StMode::S_IWUSR | StMode::S_IWGRP | StMode::S_IRGRP
}

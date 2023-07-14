//! 模拟的链接、挂载模块
//! fat32本身不支持符号链接和硬链接，两个指向相同文件的目录条目将会被chkdsk报告为交叉链接并修复
extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use axfs::api::canonicalize;
use axlog::trace;
use axsync::Mutex;

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
    pub fn is_dir(&self) -> bool {
        self.0.ends_with("/")
    }
    /// 返回是否是文件
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

/// 用户看到的文件到实际文件的映射
pub static LINK_PATH_MAP: Mutex<BTreeMap<FilePath, FilePath>> = Mutex::new(BTreeMap::new());
/// 实际文件(而不是用户文件)到链接数的映射
pub static LINK_COUNT_MAP: Mutex<BTreeMap<FilePath, usize>> = Mutex::new(BTreeMap::new());

/// 将用户提供的路径转换成实际的路径
pub fn real_path(src_path: &FilePath) -> Option<FilePath> {
    trace!("parse_file_name: {}", src_path.path());
    let map = LINK_PATH_MAP.lock();
    // 找到对应的链接
    match map.get(src_path) {
        Some(dest_path) => Some(dest_path.clone()),
        None => None,
    }
}

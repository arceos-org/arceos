//! 模拟的链接、挂载模块
//! fat32本身不支持符号链接和硬链接，两个指向相同文件的目录条目将会被chkdsk报告为交叉链接并修复
extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use axerrno::{AxError, AxResult};
use axfs::api::{canonicalize, path_exists, remove_file, FileIOType};
use axlog::{debug, info, trace};
use axsync::Mutex;

use crate::current_process;
#[allow(unused)]
pub const AT_FDCWD: usize = -100isize as usize;
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct FilePath(String);

impl FilePath {
    /// 创建一个 FilePath, 传入的 path 会被 canonicalize, 故可以是相对路径
    pub fn new(path: &str) -> AxResult<Self> {
        let new_path = canonicalize(path);
        if new_path.is_err() {
            return Err(AxError::NotFound);
        }
        let mut new_path = String::from(new_path.unwrap().trim());
        // canonicalize中没有处理末尾的空格、换行符等
        if path.ends_with("/") && !new_path.ends_with("/") {
            // 如果原始路径以 '/' 结尾，那么canonicalize后的路径也应该以 '/' 结尾
            new_path.push('/');
        }
        let new_path = real_path(&new_path);
        // assert!(!path.ends_with("/"), "path should not end with '/', link only support file");      // 链接只支持文件
        Ok(Self(new_path))
    }
    /// 获取路径
    pub fn path(&self) -> &str {
        &self.0
    }
    /// 获取所属目录
    #[allow(unused)]
    pub fn dir(&self) -> AxResult<&str> {
        if self.is_root() {
            return Ok("/");
        }
        let mut pos = if let Some(pos) = self.0.rfind("/") {
            pos
        } else {
            return Err(AxError::NotADirectory);
        };
        if pos == self.0.len() - 1 {
            // 如果是以 '/' 结尾，那么再往前找一次
            pos = if let Some(pos) = self.0[..pos].rfind("/") {
                pos
            } else {
                return Err(AxError::NotADirectory);
            };
        }
        Ok(&self.0[..=pos])
    }
    /// 获取文件/目录名
    #[allow(unused)]
    pub fn file(&self) -> AxResult<&str> {
        if self.is_root() {
            return Ok("/");
        }
        let mut pos = if let Some(pos) = self.0.rfind("/") {
            pos
        } else {
            return Err(AxError::NotFound);
        };
        if pos == self.0.len() - 1 {
            pos = if let Some(pos) = self.0[..pos].rfind("/") {
                pos
            } else {
                return Err(AxError::NotFound);
            };
        }
        Ok(&self.0[pos + 1..])
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
#[allow(unused)]
pub unsafe fn get_str_len(start: *const u8) -> usize {
    let mut ptr = start as usize;
    while *(ptr as *const u8) != 0 {
        ptr += 1;
    }
    ptr - start as usize
}

#[allow(unused)]
pub unsafe fn raw_ptr_to_ref_str(start: *const u8) -> &'static str {
    let len = get_str_len(start);
    // 因为这里直接用用户空间提供的虚拟地址来访问，所以一定能连续访问到字符串，不需要考虑物理地址是否连续
    let slice = core::slice::from_raw_parts(start, len);
    if let Ok(s) = core::str::from_utf8(slice) {
        s
    } else {
        axlog::error!("not utf8 slice");
        for c in slice {
            axlog::error!("{c} ");
        }
        axlog::error!("");
        &"p"
    }
}

/// 用户看到的文件到实际文件的映射
pub static LINK_PATH_MAP: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());
/// 实际文件(而不是用户文件)到链接数的映射
pub static LINK_COUNT_MAP: Mutex<BTreeMap<String, usize>> = Mutex::new(BTreeMap::new());

/// 将用户提供的路径转换成实际的路径
///
/// 如果在链接列表中找不到，则直接返回自己
pub fn real_path(src_path: &String) -> String {
    trace!("parse_file_name: {}", src_path);
    let map = LINK_PATH_MAP.lock();
    // 找到对应的链接
    match map.get(src_path) {
        Some(dest_path) => dest_path.clone(),
        None => {
            // 特判gcc的文件夹链接情况，即将一个文件夹前缀换成另一个文件夹前缀
            static GCC_DIR_SRC: &str =
                "/riscv64-linux-musl-native/lib/gcc/riscv64-linux-musl/11.2.1/include";
            static GCC_DIR_DST: &str = "/riscv64-linux-musl-native/include";

            static MUSL_DIR_SRC: &str = "/riscv64-linux-musl-native/riscv64-linux-musl/include";
            static MUSL_DIR_DST: &str = "/riscv64-linux-musl-native/include";
            if src_path.starts_with(GCC_DIR_SRC) {
                // 替换src为dst
                GCC_DIR_DST.to_string() + src_path.strip_prefix(GCC_DIR_SRC).unwrap()
            } else if src_path.starts_with(MUSL_DIR_SRC) {
                // 替换src为dst
                MUSL_DIR_DST.to_string() + src_path.strip_prefix(MUSL_DIR_SRC).unwrap()
            } else {
                src_path.clone()
            }
        }
    }
}

/// 删除一个链接
///
/// 如果在 map 中找不到对应链接，则什么都不做
/// 返回被删除的链接指向的文件
///
/// 现在的一个问题是，如果建立了dir1/A，并将dir2/B链接到dir1/A，那么删除dir1/A时，实际的文件不会被删除(连接数依然大于1)，只有当删除dir2/B时，实际的文件才会被删除
/// 这样的话，如果新建了dir1/A，那么就会报错(create_new)或者覆盖原文件(create)，从而影响到dir2/B
pub fn remove_link(src_path: &FilePath) -> Option<String> {
    trace!("remove_link: {}", src_path.path());
    let mut map = LINK_PATH_MAP.lock();
    // 找到对应的链接
    match map.remove(&src_path.path().to_string()) {
        Some(dest_path) => {
            // 更新链接数
            let mut count_map = LINK_COUNT_MAP.lock();
            let count = count_map.entry(dest_path.clone()).or_insert(0);
            assert!(
                count.clone() > 0,
                "before removing, the link count should > 0"
            );
            *count -= 1;
            // 如果链接数为0，那么删除文件
            if *count == 0 {
                debug!("link num down to zero, remove file: {}", dest_path);
                let _ = remove_file(dest_path.as_str());
            }
            Some(dest_path)
        }
        None => None,
    }
}

/// 获取文件的链接数
///
/// 如果文件不存在，那么返回 0
/// 如果文件存在，但是没有链接，那么返回 1
/// 如果文件存在，且有链接，那么返回链接数
pub fn get_link_count(src_path: &String) -> usize {
    trace!("get_link_count: {}", src_path);
    let map = LINK_PATH_MAP.lock();
    // 找到对应的链接
    match map.get(src_path) {
        Some(dest_path) => {
            let count_map = LINK_COUNT_MAP.lock();
            let count = count_map.get(dest_path).unwrap();
            count.clone()
        }
        None => {
            // if path_exists(src_path.path()) {
            //     1
            // } else {
            //     0
            // }
            0
        }
    }
}

/// 创建一个链接
///
/// 返回是否创建成功(已存在的链接也会返回 true)
/// 创建新文件时注意调用该函数创建链接
pub fn create_link(src_path: &FilePath, dest_path: &FilePath) -> bool {
    info!("create_link: {} -> {}", src_path.path(), dest_path.path());
    // assert!(src_path.is_file() && dest_path.is_file(), "link only support file");
    // assert_ne!(src_path.path(), dest_path.path(), "link src and dest should not be the same");  // 否则在第一步删除旧链接时可能会删除源文件
    // 检查是否是文件
    if !src_path.is_file() || !dest_path.is_file() {
        debug!("link only support file");
        return false;
    }
    // 检查被链接到的文件是否存在
    if !path_exists(dest_path.path()) {
        debug!("link dest file not exists");
        return false;
    }
    let mut map = LINK_PATH_MAP.lock();
    // 如果需要连接的文件已经存在
    if let Some(old_dest_path) = map.get(&src_path.path().to_string()) {
        // 如果不是当前链接，那么删除旧链接; 否则不做任何事
        if old_dest_path.eq(&dest_path.path().to_string()) {
            debug!("link already exists");
            return true;
        }
        remove_link(src_path);
    }
    // 创建链接
    map.insert(src_path.path().to_string(), dest_path.path().to_string());
    // 更新链接数
    let mut count_map = LINK_COUNT_MAP.lock();
    let count = count_map.entry(dest_path.path().to_string()).or_insert(0);
    *count += 1;
    true
}

pub fn deal_with_path(
    dir_fd: usize,
    path_addr: Option<*const u8>,
    force_dir: bool,
) -> Option<FilePath> {
    let process = current_process();
    let mut path = "".to_string();
    if let Some(path_addr) = path_addr {
        if path_addr.is_null() {
            axlog::debug!("path address is null");
            return None;
        }
        if process
            .manual_alloc_for_lazy((path_addr as usize).into())
            .is_ok()
        {
            // 直接访问前需要确保已经被分配
            path = unsafe { raw_ptr_to_ref_str(path_addr) }.to_string().clone();
        } else {
            axlog::debug!("path address is invalid");
            return None;
        }
    }

    if force_dir {
        path = format!("{}/", path);
    }
    if path.ends_with('.') {
        // 如果path以.或..结尾, 则加上/告诉FilePath::new它是一个目录
        path = format!("{}/", path);
    }
    // info!("path: {}", path);

    if dir_fd != AT_FDCWD {
        // 如果不是绝对路径, 且dir_fd不是AT_FDCWD, 则需要将dir_fd和path拼接起来
        let fd_table = process.fd_manager.fd_table.lock();
        if dir_fd >= fd_table.len() {
            axlog::debug!("fd index out of range");
            return None;
        }
        match fd_table[dir_fd].as_ref() {
            Some(dir) => {
                if dir.get_type() != FileIOType::DirDesc {
                    axlog::debug!("selected fd is not a dir");
                    return None;
                }
                let dir = dir.clone();
                // 有没有可能dir的尾部一定是一个/号，所以不用手工添加/
                path = format!("{}{}", dir.get_path(), path);
                axlog::debug!("handled_path: {}", path);
            }
            None => {
                axlog::debug!("fd not exist");
                return None;
            }
        }
    }
    match FilePath::new(path.as_str()) {
        Ok(path) => Some(path),
        Err(_) => None,
    }
}

//! 模拟的链接、挂载模块
//! fat32本身不支持符号链接和硬链接，两个指向相同文件的目录条目将会被chkdsk报告为交叉链接并修复


use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use log::{info, trace};
use axfs::api::{path_exists, remove_file};
use axsync::Mutex;
use crate::FilePath;


/// 用户看到的文件到实际文件的映射
static LINK_PATH_MAP: Mutex<BTreeMap<FilePath, FilePath>> = Mutex::new(BTreeMap::new());
/// 实际文件(而不是用户文件)到链接数的映射
static LINK_COUNT_MAP: Mutex<BTreeMap<FilePath, usize>> = Mutex::new(BTreeMap::new());

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

/// 检查文件名对应的链接
///
/// 如果在 map 中找不到对应链接，则返回 None
/// 相较于 real_path，这个函数会支持 gcc 的 include 目录(todo)
pub fn read_link(src_path: &FilePath) -> Option<FilePath> {
    trace!("read_link: {}", src_path.path());
    let map = LINK_PATH_MAP.lock();
    // 找到对应的链接
    match map.get(src_path) {
        Some(dest_path) => Some(dest_path.clone()),
        // 如果是链接到 gcc 的 include 目录，那么返回 gcc 的链接目录
        None => {
            static GCC_INCLUDE: &str =
                "./riscv64-linux-musl-native/lib/gcc/riscv64-linux-musl/11.2.1/include/";
            static GCC_LINK_INCLUDE: &str = "/riscv64-linux-musl-native/include/";
            if src_path.path().starts_with(GCC_INCLUDE) {
                info!("read gcc link: {}", String::from(GCC_LINK_INCLUDE) + src_path.path().strip_prefix(GCC_INCLUDE).unwrap());
                Some(FilePath::new(&(GCC_LINK_INCLUDE.to_string() + src_path.path().strip_prefix(GCC_INCLUDE).unwrap())))
            } else {
                None
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
pub fn remove_link(src_path: &FilePath) -> Option<FilePath> {
    trace!("remove_link: {}", src_path.path());
    let mut map = LINK_PATH_MAP.lock();
    // 找到对应的链接
    match map.remove(src_path) {
        Some(dest_path) => {
            // 更新链接数
            let mut count_map = LINK_COUNT_MAP.lock();
            let count = count_map.entry(dest_path.clone()).or_insert(0);
            assert!(count.clone() > 0, "before removing, the link count should > 0");
            *count -= 1;
            // 如果链接数为0，那么删除文件
            if *count == 0 {
                info!("link num down to zero, remove file: {}", dest_path.path());
                let _ = remove_file(dest_path.path());
            }
            Some(dest_path.clone())
        }
        None => None
    }
}

/// 创建一个链接
///
/// 返回是否创建成功(已存在的链接也会返回 true)
/// 创建新文件时注意调用该函数创建链接
pub fn create_link(src_path: &FilePath, dest_path: &FilePath) -> bool {
    trace!("create_link: {} -> {}", src_path.path(), dest_path.path());
    // assert!(src_path.is_file() && dest_path.is_file(), "link only support file");
    // assert_ne!(src_path.path(), dest_path.path(), "link src and dest should not be the same");  // 否则在第一步删除旧链接时可能会删除源文件
    // 检查是否是文件
    if !src_path.is_file() || !dest_path.is_file() {
        info!("link only support file");
        return false;
    }
    // 检查被链接到的文件是否存在
    if !path_exists(dest_path.path()) {
        info!("link dest file not exists");
        return false;
    }

    let mut map = LINK_PATH_MAP.lock();
    // 如果需要连接的文件已经存在
    if let Some(old_dest_path) = map.get(src_path) {
        // 如果不是当前链接，那么删除旧链接; 否则不做任何事
        if old_dest_path.equal_to(dest_path) {
            info!("link already exists");
            return true;
        }
        remove_link(src_path);
    }
    // 创建链接
    map.insert(src_path.clone(), dest_path.clone());
    // 更新链接数
    let mut count_map = LINK_COUNT_MAP.lock();
    let count = count_map.entry(dest_path.clone()).or_insert(0);
    *count += 1;
    true
}

/// 获取文件的链接数
///
/// 如果文件不存在，那么返回 0
/// 如果文件存在，但是没有链接，那么返回 1
/// 如果文件存在，且有链接，那么返回链接数
pub fn get_link_count(src_path: &FilePath) -> usize {
    trace!("get_link_count: {}", src_path.path());
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

// /// 开启系统时，初始化链接表
// pub fn init_link() {
//     trace!("init_link");
//     // 读取链接表
//     let mut map = LINK_PATH_MAP.lock();
//     let mut count_map = LINK_COUNT_MAP.lock();
//     // 将根目录下所有文件加入链接表
//     let root_path = FilePath::new("/");
//     let root_dir = root_path.open_dir().unwrap();
//     // TODO
// }





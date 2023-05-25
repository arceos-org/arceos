// //! 模拟的链接、挂载模块
// //! fat32本身不支持符号链接和硬链接，两个指向相同文件的目录条目将会被chkdsk报告为交叉链接并修复
//
// use alloc::collections::BTreeMap;
// use alloc::string::{String, ToString};
// use axerrno::AxResult;
// use axsync::Mutex;
// use crate::api::{path_exists, remove_file, canonicalize};
// use crate::fops::File;
//
// pub struct FilePath(String);    // 一个绝对路径
//
// impl FilePath {
//     /// 创建一个 FilePath，将传入的路径转换为绝对路径
//     pub fn new(path: &str) -> Self {
//         let path = String::from(canonicalize(path).unwrap().trim());     // canonicalize中没有处理末尾的空格、换行符等
//         // assert!(!path.ends_with("/"), "path should not end with '/', link only support file");      // 链接只支持文件
//         Self(path)
//     }
//
//     /// 获取路径
//     pub fn path(&self) -> &str {
//         &self.0
//     }
//
//     /// 获取所属目录
//     pub fn dir(&self) -> &str {
//         let mut pos = self.0.rfind("/").unwrap();
//         if pos == self.0.len() - 1 {
//             pos = self.0[..pos].rfind("/").unwrap();    // 如果是以 '/' 结尾，那么再往前找一次
//         }
//         &self.0[..=pos]
//     }
//
//     /// 获取文件/目录名
//     pub fn file(&self) -> &str {
//         let mut pos = self.0.rfind("/").unwrap();
//         if pos == self.0.len() - 1 {
//             pos = self.0[..pos].rfind("/").unwrap();    // 如果是以 '/' 结尾，那么再往前找一次
//         }
//         &self.0[pos + 1..]
//     }
//
//     /// 返回是否是目录
//     pub fn is_dir(&self) -> bool {
//         self.0.ends_with("/")
//     }
//
//     /// 返回是否是文件
//     pub fn is_file(&self) -> bool {
//         !self.0.ends_with("/")
//     }
//     // /// 复制自己
//     // pub fn clone(&self) -> Self {
//     //     Self(self.0.clone())
//     // }
// }
//
// impl Clone for FilePath {
//     fn clone(&self) -> Self {
//         Self(self.0.clone())
//     }
// }
//
// /// 用户看到的文件到实际文件的映射
// static LINK_PATH_MAP: Mutex<BTreeMap<FilePath, FilePath>> = Mutex::new(BTreeMap::new());
// /// 实际文件(而不是用户文件)到链接数的映射
// static LINK_COUNT_MAP: Mutex<BTreeMap<FilePath, usize>> = Mutex::new(BTreeMap::new());
//
// /// 将用户提供的路径转换成实际的路径
// pub fn real_path(src_path: &FilePath) -> Option<FilePath> {
//     trace!("parse_file_name: {}", src_path.path());
//     let map = LINK_PATH_MAP.lock();
//     // 找到对应的链接
//     match map.get(src_path) {
//         Some(dest_path) => Some(dest_path.clone()),
//         None => None,
//     }
// }
//
//
// /// 检查文件名对应的链接
// /// 如果在 map 中找不到对应链接，则返回 None
// pub fn read_link(src_path: &FilePath) -> Option<FilePath> {
//     trace!("read_link: {}", src_path.path());
//     let map = LINK_PATH_MAP.lock();
//     // 找到对应的链接
//     match map.get(src_path) {
//         Some(dest_path) => Some(dest_path.clone()),
//         // 如果是链接到 gcc 的 include 目录，那么返回 gcc 的链接目录
//         None => {
//             static GCC_INCLUDE: &str =
//                 "./riscv64-linux-musl-native/lib/gcc/riscv64-linux-musl/11.2.1/include/";
//             static GCC_LINK_INCLUDE: &str = "/riscv64-linux-musl-native/include/";
//             if src_path.starts_with(GCC_INCLUDE) {
//                 info!("read gcc link: {}", String::from(GCC_LINK_INCLUDE) + src_path.path().strip_prefix(GCC_INCLUDE).unwrap());
//                 Some(FilePath::new(&(GCC_LINK_INCLUDE.to_string() + src_path.path().strip_prefix(GCC_INCLUDE).unwrap())))
//             } else {
//                 None
//             }
//         }
//     }
// }
//
// /// 删除一个链接
// ///
// /// 如果在 map 中找不到对应链接，则什么都不做
// /// 返回被删除的链接指向的文件
// pub fn remove_link(src_path: &FilePath) -> Option<FilePath> {
//     trace!("remove_link: {}", src_path.path());
//     let mut map = LINK_PATH_MAP.lock();
//     // 找到对应的链接
//     match map.remove(src_path) {
//         Some(dest_path) => {
//             // 更新链接数
//             let mut count_map = LINK_COUNT_MAP.lock();
//             let count = count_map.entry(dest_path).or_insert(0);
//             assert!(count.clone() > 0, "Before removing, the link count should > 0");
//             *count -= 1;
//             // 如果链接数为0，那么删除文件
//             if *count == 0 {
//                 let _ = remove_file(dest_path.path());
//             }
//             Some(dest_path.clone())
//         }
//         None => None
//     }
// }
//
//
// /// 创建一个链接
// ///
// /// 返回是否创建成功
// pub fn create_link(src_path: &FilePath, dest_path: &FilePath) -> bool {
//     trace!("create_link: {} -> {}", src_path.path(), dest_path.path());
//     // assert!(src_path.is_file() && dest_path.is_file(), "link only support file");
//     // assert_ne!(src_path.path(), dest_path.path(), "link src and dest should not be the same");  // 否则在第一步删除旧链接时可能会删除源文件
//     // 检查是否是文件
//     if !src_path.is_file() || !dest_path.is_file() {
//         info!("link only support file");
//         return false;
//     }
//     // 检查被链接到的文件是否存在
//     if !path_exists(dest_path.path()) {
//         info!("link dest file not exists");
//         return false;
//     }
//
//     let mut map = LINK_PATH_MAP.lock();
//     // 如果需要连接的文件已经存在，那么先删除(直接insert不能更新value)
//     if map.contains_key(src_path) {
//         remove_link(src_path);
//     }
//     // 创建链接
//     map.insert(src_path.clone(), dest_path.clone());
//     // 更新链接数
//     let mut count_map = LINK_COUNT_MAP.lock();
//     let count = count_map.entry(dest_path.clone()).or_insert(0);
//     *count += 1;
//     true
// }
//
// /// 获取文件的链接数
// ///
// /// 如果文件不存在，那么返回 0
// /// 如果文件存在，但是没有链接，那么返回 1
// /// 如果文件存在，且有链接，那么返回链接数
// pub fn get_link_count(src_path: &FilePath) -> usize {
//     trace!("get_link_count: {}", src_path.path());
//     let map = LINK_PATH_MAP.lock();
//     // 找到对应的链接
//     match map.get(src_path) {
//         Some(dest_path) => {
//             let count_map = LINK_COUNT_MAP.lock();
//             let count = count_map.get(dest_path).unwrap();
//             count.clone()
//         }
//         None => {
//             if path_exists(src_path.path()) {
//                 1
//             } else {
//                 0
//             }
//         }
//     }
// }
//
//
//

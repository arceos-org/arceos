//! 模拟的链接、挂载模块
//! fat32本身不支持符号链接和硬链接，两个指向相同文件的目录条目将会被chkdsk报告为交叉链接并修复
extern crate alloc;
use alloc::string::{String, ToString};
use axfs::api::{path_exists, remove_file};
use axlog::{debug, info, trace};
use axprocess::link::{FilePath, LINK_COUNT_MAP, LINK_PATH_MAP};

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

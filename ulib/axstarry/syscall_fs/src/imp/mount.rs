use axprocess::{
    current_process,
    link::{deal_with_path, raw_ptr_to_ref_str, AT_FDCWD},
};
use syscall_utils::{SyscallError, SyscallResult};

// use super::{deal_with_path, AT_FDCWD};
use crate::ctype::mount::{check_mounted, mount_fat_fs, umount_fat_fs};
extern crate alloc;
use alloc::string::ToString;
use axlog::debug;
/// 功能：挂载文件系统；
/// 输入：
///   - special: 挂载设备；
///   - dir: 挂载点；       经过实测，发现dir可以是绝对路径，也可以是相对路径，甚至可以是 . 或 ..
///   - fs_type: 挂载的文件系统类型；
///   - flags: 挂载参数；
///   - data: 传递给文件系统的字符串参数，可为NULL；
/// 返回值：成功返回0，失败返回-1
pub fn syscall_mount(
    special: *const u8,
    dir: *const u8,
    fs_type: *const u8,
    _flags: usize,
    _data: *const u8,
) -> SyscallResult {
    let device_path = deal_with_path(AT_FDCWD, Some(special), false).unwrap();
    // 这里dir必须以"/"结尾，但在shell中输入时，不需要以"/"结尾
    let mount_path = deal_with_path(AT_FDCWD, Some(dir), true).unwrap();

    let process = current_process();
    if process
        .manual_alloc_for_lazy((fs_type as usize).into())
        .is_err()
    {
        return Err(SyscallError::EINVAL);
    }

    let fs_type = unsafe { raw_ptr_to_ref_str(fs_type).to_string() };
    let mut _data_str = "".to_string();
    if !_data.is_null() {
        if process
            .manual_alloc_for_lazy((_data as usize).into())
            .is_err()
        {
            return Err(SyscallError::EINVAL);
        }
        // data可以为NULL, 必须判断, 否则会panic, 发生LoadPageFault
        _data_str = unsafe { raw_ptr_to_ref_str(_data) }.to_string();
    }
    if device_path.is_dir() {
        debug!("device_path should not be a dir");
        return Err(SyscallError::EPERM);
    }
    if !mount_path.is_dir() {
        debug!("mount_path should be a dir");
        return Err(SyscallError::EPERM);
    }

    // 如果mount_path不存在，则创建
    if !axfs::api::path_exists(mount_path.path()) {
        if let Err(e) = axfs::api::create_dir(mount_path.path()) {
            debug!("create mount path error: {:?}", e);
            return Err(SyscallError::EPERM);
        }
    }

    if fs_type != "vfat" {
        debug!("fs_type can only be vfat.");
        return Err(SyscallError::EPERM);
    }
    // 检查挂载点路径是否存在
    if !axfs::api::path_exists(mount_path.path()) {
        debug!("mount path not exist");
        return Err(SyscallError::EPERM);
    }
    // 查挂载点是否已经被挂载
    if check_mounted(&mount_path) {
        debug!("mount path includes mounted fs");
        return Err(SyscallError::EPERM);
    }
    // 挂载
    if !mount_fat_fs(&device_path, &mount_path) {
        debug!("mount error");
        return Err(SyscallError::EPERM);
    }

    Ok(0)
}

/// 功能：卸载文件系统；
/// 输入：指定卸载目录，卸载参数；
/// 返回值：成功返回0，失败返回-1；
pub fn syscall_umount(dir: *const u8, flags: usize) -> SyscallResult {
    let mount_path = deal_with_path(AT_FDCWD, Some(dir), true).unwrap();

    if flags != 0 {
        debug!("flags unimplemented");
        return Err(SyscallError::EPERM);
    }

    // 检查挂载点路径是否存在
    if !axfs::api::path_exists(mount_path.path()) {
        debug!("mount path not exist");
        return Err(SyscallError::EPERM);
    }
    // 从挂载点中删除
    if !umount_fat_fs(&mount_path) {
        debug!("umount error");
        return Err(SyscallError::EPERM);
    }

    Ok(0)
}

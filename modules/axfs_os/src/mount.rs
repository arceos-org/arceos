use alloc::vec::Vec;
use log::{debug, info};
use axfs::api::path_exists;
use axsync::Mutex;
use crate::FilePath;
use crate::link::{real_path};

/// 挂载的文件系统。
/// 目前"挂载"的语义是，把一个文件当作文件系统读写
pub struct MountedFs {
    //pub inner: Arc<Mutex<FATFileSystem>>,
    pub device: FilePath,
    pub mnt_dir: FilePath,
}

impl MountedFs {
    pub fn new(device: &FilePath, mnt_dir: &FilePath) -> Self {
        assert!(device.is_file() && mnt_dir.is_dir(), "device must be a file and mnt_dir must be a dir");
        Self {
            device: device.clone(),
            mnt_dir: mnt_dir.clone(),
        }
    }

    pub fn device(&self) -> FilePath {
        self.device.clone()
    }

    pub fn mnt_dir(&self) -> FilePath {
        self.mnt_dir.clone()
    }
}

/// 已挂载的文件系统(设备)。
/// 注意启动时的文件系统不在这个 vec 里，它在 mod.rs 里。
static MOUNTED: Mutex<Vec<MountedFs>> = Mutex::new(Vec::new());

/// 挂载一个fatfs类型的设备
pub fn mount_fat_fs(device_path: &FilePath, mount_path: &FilePath) -> bool {
    // // device_path需要链接转换, mount_path不需要, 因为目前目录没有链接  // 暂时只有Open过的文件会加入到链接表，所以这里先不转换
    // debug!("mounting {} to {}", device_path.path(), mount_path.path());
    // if let Some(true_device_path) = real_path(device_path) {
    if path_exists(mount_path.path()) {
        MOUNTED.lock().push(MountedFs::new(
            device_path,
            mount_path,
        ));
        info!("mounted {} to {}", device_path.path(), mount_path.path());
        return true;
    }
    // }
    info!("mount failed: {} to {}", device_path.path(), mount_path.path());
    false
}

/// 卸载一个fatfs类型的设备
pub fn umount_fat_fs(mount_path: &FilePath) -> bool {
    let mut mounted = MOUNTED.lock();
    let mut i = 0;
    while i < mounted.len() {
        if mounted[i].mnt_dir().equal_to(mount_path) {
            mounted.remove(i);
            info!("umounted {}", mount_path.path());
            return true;
        }
        i += 1;
    }
    info!("umount failed: {}", mount_path.path());
    false
}

/// 检查一个路径是否已经被挂载
pub fn check_mounted(path: &FilePath) -> bool {
    let mounted = MOUNTED.lock();
    for m in mounted.iter() {
        if path.start_with(&m.mnt_dir()) {
            debug!("{} is mounted", path.path());
            return true;
        }
    }
    false
}
#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use axerror::{AxError, AxResult};
use lazy_init::LazyInit;
use spinlock::SpinNoIrq;
use vfs::dentry::DirEntry;
use vfs::file::{
    vfs_llseek, vfs_mkdir, vfs_open_file, vfs_read_file, vfs_readdir, vfs_write_file, File,
};
use vfs::info::{ProcessFs, ProcessFsInfo, VfsTime};
use vfs::mount::VfsMount;
use vfs::mount_rootfs;

pub use vfs::file::{FileMode, OpenFlags, SeekFrom};

type Mutex<T> = SpinNoIrq<T>;

pub static ROOT_MNT: LazyInit<Mutex<Arc<VfsMount>>> = LazyInit::new();
pub static ROOT_DIR: LazyInit<Mutex<Arc<DirEntry>>> = LazyInit::new();
pub static NOW_MNT: LazyInit<Mutex<Arc<VfsMount>>> = LazyInit::new();
pub static NOW_DIR: LazyInit<Mutex<Arc<DirEntry>>> = LazyInit::new();

/// This function is used to initialize the rootfs
///
/// Because there is no process, we used some static variables to store the root/current directory/mount info
pub fn init_vfs() {
    // init the rootfs
    let root_mnt = mount_rootfs();
    ROOT_MNT.init_by(Mutex::new(root_mnt.clone()));
    ROOT_DIR.init_by(Mutex::new(root_mnt.root.clone()));
    NOW_MNT.init_by(Mutex::new(root_mnt.clone()));
    NOW_DIR.init_by(Mutex::new(root_mnt.root.clone()));
}

pub fn open(path: &str, flag: OpenFlags, mode: FileMode) -> Option<Arc<File>> {
    let file = vfs_open_file::<VfsProvider>(path, flag, mode);
    if file.is_err() {
        return None;
    }
    Some(file.unwrap())
}

pub fn read(file: Arc<File>, buf: &mut [u8]) -> AxResult<usize> {
    let offset = file.access_inner().f_pos;
    let r =
        vfs_read_file::<VfsProvider>(file.clone(), buf, offset as u64).map_err(|_| AxError::Io)?;
    Ok(r)
}

pub fn write(file: Arc<File>, buf: &[u8]) -> AxResult<usize> {
    let offset = file.access_inner().f_pos;
    let r = vfs_write_file::<VfsProvider>(file, buf, offset as u64).map_err(|_| AxError::Io)?;
    Ok(r)
}

pub fn lseek(file: Arc<File>, seek: SeekFrom) -> AxResult<usize> {
    let res = vfs_llseek(file, seek).map_err(|_| AxError::Io)?;
    Ok(res as usize)
}

pub fn mkdir(path: &str, mode: FileMode) -> AxResult<()> {
    vfs_mkdir::<VfsProvider>(path, mode).map_err(|_| AxError::Io)?;
    Ok(())
}

pub fn list(path: &str) -> AxResult<Vec<String>> {
    let file = vfs_open_file::<VfsProvider>(
        path,
        OpenFlags::O_RDWR | OpenFlags::O_DIRECTORY,
        FileMode::FMODE_READ,
    )
    .map_err(|_| AxError::Io)?;
    let mut res = Vec::new();
    vfs_readdir(file).map_err(|_| AxError::Io)?.for_each(|x| {
        res.push(x);
    });
    Ok(res)
}

struct VfsProvider;
impl ProcessFs for VfsProvider {
    fn get_fs_info() -> ProcessFsInfo {
        let root_mnt = ROOT_MNT.lock().clone();
        let root_dir = ROOT_DIR.lock().clone();
        let now_mnt = NOW_MNT.lock().clone();
        let now_dir = NOW_DIR.lock().clone();
        ProcessFsInfo::new(root_mnt, root_dir, now_dir, now_mnt)
    }
    fn check_nested_link() -> bool {
        false
    }

    fn update_link_data() {}

    fn max_link_count() -> u32 {
        10
    }

    fn current_time() -> VfsTime {
        VfsTime {
            year: 2023,
            month: 3,
            day: 3,
            hour: 3,
            minute: 3,
            second: 3,
        }
    }
}

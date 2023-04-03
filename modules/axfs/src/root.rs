use alloc::{sync::Arc, vec::Vec};
use axerrno::{ax_err, AxResult};
use axfs_vfs::{VfsNodeRef, VfsOps};
use lazy_init::LazyInit;

use crate::fs;

#[cfg(feature = "fatfs")]
type MainFileSystem = fs::fatfs::FatFileSystem;

struct MountPoint {
    path: &'static str,
    fs: Arc<dyn VfsOps>,
}

struct RootDirectory {
    mounts: Vec<MountPoint>,
}

static MAIN_FS: LazyInit<Arc<MainFileSystem>> = LazyInit::new();
static ROOT_DIR: LazyInit<RootDirectory> = LazyInit::new();

impl MountPoint {
    pub fn new(path: &'static str, fs: Arc<dyn VfsOps>) -> Self {
        Self { path, fs }
    }
}

impl Drop for MountPoint {
    fn drop(&mut self) {
        self.fs.umount().ok();
    }
}

impl RootDirectory {
    pub const fn new() -> Self {
        Self { mounts: Vec::new() }
    }

    pub fn mount(&mut self, path: &'static str, fs: Arc<dyn VfsOps>) -> AxResult {
        if !path.starts_with('/') {
            return ax_err!(InvalidInput, "mount path must start with '/'");
        }
        if self.mounts.iter().any(|mp| mp.path == path) {
            return ax_err!(InvalidInput, "mount point already exists");
        }
        fs.mount(path)?;
        self.mounts.push(MountPoint::new(path, fs));
        Ok(())
    }

    pub fn _umount(&mut self, path: &str) {
        self.mounts.retain(|mp| mp.path != path);
    }

    pub fn lookup(&self, path: &str) -> AxResult<VfsNodeRef> {
        debug!("lookup at root: {}", path);

        let mut idx = 0;
        let mut max_len = 0;

        // Find the filesystem that has the longest mounted path match
        // TODO: more efficient, e.g. trie
        for (i, mp) in self.mounts.iter().enumerate() {
            // skip the first '/'
            if path.starts_with(mp.path) && mp.path.len() > max_len {
                max_len = mp.path.len();
                idx = i;
            }
        }

        let mp = &self.mounts[idx];
        let node = mp.fs.root_dir().lookup(&path[max_len..])?;
        Ok(node)
    }
}

pub(crate) fn init_rootfs(disk: crate::dev::Disk) {
    #[cfg(feature = "fatfs")]
    let main_fs = fs::fatfs::FatFileSystem::new(disk);

    MAIN_FS.init_by(Arc::new(main_fs));
    MAIN_FS.init();

    let mut root_dir = RootDirectory::new();
    root_dir.mount("/", MAIN_FS.clone()).unwrap();

    #[cfg(feature = "devfs")]
    {
        let null = fs::devfs::NullDev;
        let zero = fs::devfs::ZeroDev;
        let bar = fs::devfs::ZeroDev;
        let mut foo_dir = fs::devfs::DirNode::new();
        foo_dir.add("bar", Arc::new(bar));

        let mut devfs = fs::devfs::DeviceFileSystem::new();
        devfs.add("null", Arc::new(null));
        devfs.add("zero", Arc::new(zero));
        devfs.add("foo", Arc::new(foo_dir));
        root_dir
            .mount("/dev", Arc::new(devfs))
            .expect("failed to mount devfs at /dev");
    }

    ROOT_DIR.init_by(root_dir);
}

pub(crate) fn lookup(path: &str) -> AxResult<VfsNodeRef> {
    ROOT_DIR.lookup(path)
}

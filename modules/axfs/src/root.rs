//! Root directory of the filesystem
//!
//! TODO: it doesn't work very well if the mount points have containment relationships.

use alloc::{string::String, sync::Arc, vec::Vec};
use axerrno::{ax_err, AxResult};
use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsOps, VfsResult};
use axsync::Mutex;
use lazy_init::LazyInit;

use crate::fs;

static CURRENT_DIR_PATH: Mutex<String> = Mutex::new(String::new());
static CURRENT_DIR: LazyInit<Mutex<VfsNodeRef>> = LazyInit::new();

#[cfg(feature = "fatfs")]
type MainFileSystem = fs::fatfs::FatFileSystem;

struct MountPoint {
    path: &'static str,
    fs: Arc<dyn VfsOps>,
}

struct RootDirectory {
    main_fs: Arc<dyn VfsOps>,
    mounts: Vec<MountPoint>,
}

static MAIN_FS: LazyInit<Arc<MainFileSystem>> = LazyInit::new();
static ROOT_DIR: LazyInit<Arc<RootDirectory>> = LazyInit::new();

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
    pub const fn new(main_fs: Arc<dyn VfsOps>) -> Self {
        Self {
            main_fs,
            mounts: Vec::new(),
        }
    }

    pub fn mount(&mut self, path: &'static str, fs: Arc<dyn VfsOps>) -> AxResult {
        if path == "/" {
            return ax_err!(InvalidInput, "cannot mount root filesystem");
        }
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

    fn lookup_mounted_fs<F, T>(&self, path: &str, f: F) -> AxResult<T>
    where
        F: FnOnce(Arc<dyn VfsOps>, &str) -> AxResult<T>,
    {
        debug!("lookup at root: {}", path);
        let path = path.trim_matches('/');
        if let Some(rest) = path.strip_prefix("./") {
            return self.lookup_mounted_fs(rest, f);
        }

        let mut idx = 0;
        let mut max_len = 0;

        // Find the filesystem that has the longest mounted path match
        // TODO: more efficient, e.g. trie
        for (i, mp) in self.mounts.iter().enumerate() {
            // skip the first '/'
            if path.starts_with(&mp.path[1..]) && mp.path.len() - 1 > max_len {
                max_len = mp.path.len() - 1;
                idx = i;
            }
        }

        if max_len == 0 {
            f(self.main_fs.clone(), path) // not matched any mount point
        } else {
            f(self.mounts[idx].fs.clone(), &path[max_len..]) // matched at `idx`
        }
    }
}

impl VfsNodeOps for RootDirectory {
    axfs_vfs::impl_vfs_dir_default! {}

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        self.main_fs.root_dir().get_attr()
    }

    fn lookup(self: Arc<Self>, path: &str) -> VfsResult<VfsNodeRef> {
        self.lookup_mounted_fs(path, |fs, rest_path| fs.root_dir().lookup(rest_path))
    }
}

pub(crate) fn init_rootfs(disk: crate::dev::Disk) {
    #[cfg(feature = "fatfs")]
    let main_fs = fs::fatfs::FatFileSystem::new(disk);

    MAIN_FS.init_by(Arc::new(main_fs));
    MAIN_FS.init();

    let mut root_dir = RootDirectory::new(MAIN_FS.clone());

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

    ROOT_DIR.init_by(Arc::new(root_dir));
    CURRENT_DIR.init_by(Mutex::new(ROOT_DIR.clone()));
    *CURRENT_DIR_PATH.lock() = "/".into();
}

fn parent_node_of(path: &str) -> VfsNodeRef {
    if path.starts_with('/') {
        ROOT_DIR.clone()
    } else {
        CURRENT_DIR.lock().clone()
    }
}

pub(crate) fn lookup(path: &str) -> AxResult<VfsNodeRef> {
    if path.is_empty() {
        return ax_err!(NotFound);
    }
    let node = parent_node_of(path).lookup(path)?;
    if path.ends_with('/') && !node.get_attr()?.is_dir() {
        ax_err!(NotADirectory)
    } else {
        Ok(node)
    }
}

pub(crate) fn current_dir() -> AxResult<String> {
    Ok(CURRENT_DIR_PATH.lock().clone())
}

pub(crate) fn set_current_dir(path: &str) -> AxResult {
    let node = lookup(path)?;
    let attr = node.get_attr()?;
    if !attr.is_dir() {
        ax_err!(NotADirectory)
    } else {
        let mut path = if path.starts_with('/') {
            path.into()
        } else {
            CURRENT_DIR_PATH.lock().clone() + path // TODO: canonicalize
        };
        if !path.ends_with('/') {
            path += "/";
        }
        *CURRENT_DIR.lock() = node;
        *CURRENT_DIR_PATH.lock() = path;
        Ok(())
    }
}

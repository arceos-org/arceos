use axerrno::ax_err;
use axerrno::AxResult;
use axfs_vfs::{VfsDirEntry, VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType, VfsOps, VfsResult};
use axsync::Mutex;
use lazy_init::LazyInit;

use crate::fs;
use alloc::{string::String, sync::Arc, vec::Vec};

static CURRENT_DIR_PATH: Mutex<String> = Mutex::new(String::new());
static CURRENT_DIR: LazyInit<Mutex<VfsNodeRef>> = LazyInit::new();

#[cfg(feature = "fatfs")]
type MainFileSystem = fs::fatfs::FatFileSystem;

struct MountPoint {
    path: &'static str,
    fs: Arc<dyn VfsOps>,
}

struct RootDirectory(Vec<MountPoint>);

static MAIN_FS: LazyInit<Arc<MainFileSystem>> = LazyInit::new();
static ROOT_DIR: LazyInit<Arc<RootDirectory>> = LazyInit::new();

impl MountPoint {
    pub fn new(path: &'static str, fs: Arc<dyn VfsOps>) -> AxResult<Self> {
        fs.mount(path)?;
        Ok(Self { path, fs })
    }
}

impl Drop for MountPoint {
    fn drop(&mut self) {
        self.fs.umount().ok();
    }
}

impl RootDirectory {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn mount(&mut self, path: &'static str, fs: Arc<dyn VfsOps>) {
        assert!(path.starts_with('/'));
        match MountPoint::new(path, fs) {
            Ok(mp) => self.0.push(mp),
            Err(e) => error!("failed to mount filesystem at {:?}: {:?}", path, e),
        }
    }

    pub fn _umount(&mut self, path: &str) {
        self.0.retain(|mp| mp.path != path);
    }

    fn lookup_mounted_fs<F, T>(&self, path: &str, f: F) -> AxResult<T>
    where
        F: FnOnce(Arc<dyn VfsOps>, &str) -> AxResult<T>,
    {
        let path = path.trim_matches('/');
        debug!("lookup at root: /{}", path);

        let mut idx = 0;
        let mut max_len = 0;

        // Find the filesystem that has the longest mounted path match
        // TODO: more efficient, e.g. trie
        for (i, mp) in self.0.iter().enumerate() {
            // skip '/'
            if path.starts_with(&mp.path[1..]) && mp.path.len() > max_len {
                max_len = mp.path.len();
                idx = i;
            }
        }
        assert!(max_len > 0);

        let mp = &self.0[idx];
        f(mp.fs.clone(), &path[max_len - 1..])
    }
}

impl VfsNodeOps for RootDirectory {
    axfs_vfs::impl_vfs_dir_default! {}

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        MAIN_FS.root_dir().get_attr()
    }

    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        MAIN_FS.root_dir().read_dir(start_idx, dirents)
    }

    fn lookup(self: Arc<Self>, path: &str) -> VfsResult<VfsNodeRef> {
        self.lookup_mounted_fs(path, |fs, rest_path| fs.root_dir().lookup(rest_path))
    }

    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult<VfsNodeRef> {
        self.lookup_mounted_fs(path, |fs, rest_path| fs.root_dir().create(rest_path, ty))
    }

    fn remove(&self, path: &str) -> VfsResult {
        self.lookup_mounted_fs(path, |fs, rest_path| fs.root_dir().remove(rest_path))
    }
}

pub(crate) fn init_rootfs(disk: crate::dev::Disk) {
    #[cfg(feature = "fatfs")]
    let main_fs = fs::fatfs::FatFileSystem::new(disk);

    MAIN_FS.init_by(Arc::new(main_fs));
    MAIN_FS.init();

    let mut root_dir = RootDirectory::new();
    root_dir.mount("/", MAIN_FS.clone());

    #[cfg(feature = "devfs")]
    {
        // TODO: mkdir "/dev"
        let null = fs::devfs::NullDev;
        let zero = fs::devfs::ZeroDev;
        let bar = fs::devfs::ZeroDev;
        let mut foo_dir = fs::devfs::DirNode::new();
        foo_dir.add("bar", Arc::new(bar));

        let mut devfs = fs::devfs::DeviceFileSystem::new();
        devfs.add("null", Arc::new(null));
        devfs.add("zero", Arc::new(zero));
        devfs.add("foo", Arc::new(foo_dir));
        root_dir.mount("/dev", Arc::new(devfs));
    }

    ROOT_DIR.init_by(Arc::new(root_dir));
    CURRENT_DIR.init_by(Mutex::new(ROOT_DIR.clone()));
    *CURRENT_DIR_PATH.lock() = "/".into();
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

fn dir_of(path: &str) -> VfsNodeRef {
    if path.starts_with('/') {
        ROOT_DIR.clone()
    } else {
        CURRENT_DIR.lock().clone()
    }
}

pub(crate) fn lookup(path: &str) -> AxResult<VfsNodeRef> {
    dir_of(path).lookup(path)
}

pub(crate) fn create_file(path: &str) -> AxResult<VfsNodeRef> {
    dir_of(path).create(path, VfsNodeType::File)
}

pub(crate) fn _mkdir(path: &str) -> AxResult<VfsNodeRef> {
    dir_of(path).create(path, VfsNodeType::Dir)
}

pub(crate) fn _rm(path: &str) -> AxResult {
    dir_of(path).remove(path)
}

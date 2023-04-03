use axerrno::AxResult;
use axfs_vfs::{VfsNodeRef, VfsOps};
use lazy_init::LazyInit;

use crate::fs;
use alloc::{sync::Arc, vec::Vec};

#[cfg(feature = "fatfs")]
type MainFileSystem = fs::fatfs::FatFileSystem;

struct MountPoint {
    path: &'static str,
    fs: Arc<dyn VfsOps>,
}

struct RootFileSystem(Vec<MountPoint>);

static MAIN_FS: LazyInit<Arc<MainFileSystem>> = LazyInit::new();
static ROOT_FS: LazyInit<RootFileSystem> = LazyInit::new();

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

impl RootFileSystem {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn mount(&mut self, path: &'static str, fs: Arc<dyn VfsOps>) {
        match MountPoint::new(path, fs) {
            Ok(mp) => self.0.push(mp),
            Err(e) => error!("failed to mount filesystem at {:?}: {:?}", path, e),
        }
    }

    pub fn _umount(&mut self, path: &str) {
        self.0.retain(|mp| mp.path != path);
    }

    pub fn lookup(&self, path: &str) -> AxResult<VfsNodeRef> {
        let mut idx = 0;
        let mut max_len = 0;

        // Find the filesystem that has the longest mounted path match
        // TODO: more efficient, e.g. trie
        for (i, mp) in self.0.iter().enumerate() {
            if path.starts_with(mp.path) && mp.path.len() > max_len {
                max_len = mp.path.len();
                idx = i;
            }
        }

        let mp = &self.0[idx];
        let rest = path.strip_prefix(mp.path).unwrap();
        let node = mp.fs.root_dir().lookup(rest)?;
        Ok(node)
    }
}

pub(crate) fn init_rootfs(disk: crate::dev::Disk) {
    #[cfg(feature = "fatfs")]
    let main_fs = fs::fatfs::FatFileSystem::new(disk);

    MAIN_FS.init_by(Arc::new(main_fs));
    MAIN_FS.init();

    let mut root_fs = RootFileSystem::new();
    root_fs.mount("/", MAIN_FS.clone());

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
        root_fs.mount("/dev", Arc::new(devfs));
    }

    ROOT_FS.init_by(root_fs);
}

pub(crate) fn lookup(path: &str) -> AxResult<VfsNodeRef> {
    ROOT_FS.lookup(path)
}

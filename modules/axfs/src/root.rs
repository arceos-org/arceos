//! Root directory of the filesystem
//!
//! TODO: it doesn't work very well if the mount points have containment relationships.

use alloc::{string::String, sync::Arc, vec::Vec};
use axerrno::{ax_err, AxError, AxResult};
use axfs_vfs::{VfsError, VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType, VfsOps, VfsResult};
use axsync::Mutex;
use lazy_init::LazyInit;

use crate::{api::FileType, fs};

static CURRENT_DIR_PATH: Mutex<String> = Mutex::new(String::new());
static CURRENT_DIR: LazyInit<Mutex<VfsNodeRef>> = LazyInit::new();

struct MountPoint {
    path: &'static str,
    fs: Arc<dyn VfsOps>,
}

struct RootDirectory {
    main_fs: Arc<dyn VfsOps>,
    mounts: Vec<MountPoint>,
}

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

impl Drop for RootDirectory {
    fn drop(&mut self) {
        self.close();
    }
}

impl RootDirectory {
    pub const fn new(main_fs: Arc<dyn VfsOps>) -> Self {
        Self {
            main_fs,
            mounts: Vec::new(),
        }
    }

    fn close(&self) {
        debug!("Close RootDirectory");
        let _ = self.main_fs.umount();
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
        // create the mount point in the main filesystem if it does not exist
        let cres = self.main_fs.root_dir().create(path, FileType::Dir);
        if cres.is_err() {
            match cres {
                Err(AxError::AlreadyExists) => (),
                Err(_) => {
                    return cres;
                }
                _ => unreachable!(),
            }
        }
        fs.mount(path, self.main_fs.root_dir().lookup(path)?)?;
        self.mounts.push(MountPoint::new(path, fs));
        Ok(())
    }

    pub fn _umount(&mut self, path: &str) {
        self.mounts.retain(|mp| mp.path != path);
    }

    pub fn contains(&self, path: &str) -> bool {
        self.mounts.iter().any(|mp| mp.path == path)
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

    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult {
        self.lookup_mounted_fs(path, |fs, rest_path| {
            if rest_path.is_empty() {
                Ok(()) // already exists
            } else {
                fs.root_dir().create(rest_path, ty)
            }
        })
    }

    fn remove(&self, path: &str, recursive: bool) -> VfsResult {
        self.lookup_mounted_fs(path, |fs, rest_path| {
            if rest_path.is_empty() {
                ax_err!(PermissionDenied) // cannot remove mount points
            } else {
                fs.root_dir().remove(rest_path, recursive)
            }
        })
    }

    fn link(&self, name: &str, handle: &axfs_vfs::LinkHandle) -> VfsResult {
        self.lookup_mounted_fs(name, |fs, rest_path| {
            if rest_path.is_empty() {
                ax_err!(PermissionDenied) // cannot remove mount points
            } else {
                fs.root_dir().link(rest_path, handle)
            }
        })
    }

    fn symlink(&self, name: &str, spath: &str) -> VfsResult {
        self.lookup_mounted_fs(name, |fs, rest_path| {
            if rest_path.is_empty() {
                ax_err!(PermissionDenied) // cannot remove mount points
            } else {
                fs.root_dir().symlink(rest_path, spath)
            }
        })
    }
}

pub(crate) fn init_rootfs(disk: crate::dev::Disk) {
    cfg_if::cfg_if! {
        if #[cfg(feature = "myfs")] { // override the default filesystem
            let main_fs = fs::myfs::new_myfs(disk);
        } else if #[cfg(feature = "ext2fs")] {
            static EXT2_FS: LazyInit<Arc<fs::ext2fs::Ext2FileSystem>> = LazyInit::new();
            EXT2_FS.init_by(Arc::new(fs::ext2fs::Ext2FileSystem::new(disk)));
            EXT2_FS.init();
            let main_fs = EXT2_FS.clone();
        } else if #[cfg(feature = "fatfs")] {
            static FAT_FS: LazyInit<Arc<fs::fatfs::FatFileSystem>> = LazyInit::new();
            FAT_FS.init_by(Arc::new(fs::fatfs::FatFileSystem::new(disk)));
            FAT_FS.init();
            let main_fs = FAT_FS.clone();
        }
    }

    let mut root_dir = RootDirectory::new(main_fs);

    #[cfg(feature = "devfs")]
    {
        let null = fs::devfs::NullDev;
        let zero = fs::devfs::ZeroDev;
        let bar = fs::devfs::ZeroDev;
        let devfs = fs::devfs::DeviceFileSystem::new();
        let foo_dir = devfs.mkdir("foo");
        devfs.add("null", Arc::new(null));
        devfs.add("zero", Arc::new(zero));
        foo_dir.add("bar", Arc::new(bar));

        root_dir
            .mount("/dev", Arc::new(devfs))
            .expect("failed to mount devfs at /dev");
    }

    #[cfg(feature = "ramfs")]
    {
        let ramfs = fs::ramfs::RamFileSystem::new();
        root_dir
            .mount("/tmp", Arc::new(ramfs))
            .expect("failed to mount ramfs at /tmp");
    }

    ROOT_DIR.init_by(Arc::new(root_dir));
    CURRENT_DIR.init_by(Mutex::new(ROOT_DIR.clone()));
    *CURRENT_DIR_PATH.lock() = "/".into();
}

fn parent_node_of(dir: Option<&VfsNodeRef>, path: &str) -> VfsNodeRef {
    if path.starts_with('/') {
        ROOT_DIR.clone()
    } else {
        dir.cloned().unwrap_or_else(|| CURRENT_DIR.lock().clone())
    }
}

pub(crate) fn absolute_path(path: &str) -> AxResult<String> {
    if path.starts_with('/') {
        Ok(axfs_vfs::path::canonicalize(path))
    } else {
        let path = CURRENT_DIR_PATH.lock().clone() + path;
        Ok(axfs_vfs::path::canonicalize(&path))
    }
}

pub(crate) fn lookup(dir: Option<&VfsNodeRef>, path: &str) -> AxResult<VfsNodeRef> {
    // if path.is_empty() {
    //     return ax_err!(NotFound);
    // }
    // let node = parent_node_of(dir, path).lookup(path)?;
    // if path.ends_with('/') && !node.get_attr()?.is_dir() {
    //     ax_err!(NotADirectory)
    // } else {
    //     Ok(node)
    // }
    lookup_symbolic(dir, path, true)
}

pub(crate) fn create_file(dir: Option<&VfsNodeRef>, path: &str) -> AxResult<VfsNodeRef> {
    if path.is_empty() {
        return ax_err!(NotFound);
    } else if path.ends_with('/') {
        return ax_err!(NotADirectory);
    }
    // let parent = parent_node_of(dir, path);
    // parent.create(path, VfsNodeType::File)?;
    // parent.lookup(path)
    let (parent, child_name) = lookup_parent(dir, path)?;
    parent.create(&child_name, VfsNodeType::File)?;
    parent.lookup(&child_name)
}

pub(crate) fn create_dir(dir: Option<&VfsNodeRef>, path: &str) -> AxResult {
    match lookup(dir, path) {
        Ok(_) => ax_err!(AlreadyExists),
        Err(AxError::NotFound) => {
            let (parent, child_name) = lookup_parent(dir, path)?;
            parent.create(&child_name, VfsNodeType::Dir)?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub(crate) fn remove_file(dir: Option<&VfsNodeRef>, path: &str) -> AxResult {
    let node = lookup_symbolic(dir, path, false)?;
    let attr = node.get_attr()?;
    if attr.is_dir() {
        ax_err!(IsADirectory)
    } else if !attr.perm().owner_writable() {
        ax_err!(PermissionDenied)
    } else {
        let (parent, child_name) = lookup_parent(dir, path)?;
        parent.remove(&child_name, false)?;
        Ok(())
    }
}

pub(crate) fn remove_dir(dir: Option<&VfsNodeRef>, path: &str, recursive: bool) -> AxResult {
    if path.is_empty() {
        return ax_err!(NotFound);
    }
    let path_check = path.trim_matches('/');
    if path_check.is_empty() {
        return ax_err!(DirectoryNotEmpty); // rm -d '/'
    } else if path_check == "."
        || path_check == ".."
        || path_check.ends_with("/.")
        || path_check.ends_with("/..")
    {
        return ax_err!(InvalidInput);
    }
    if ROOT_DIR.contains(&absolute_path(path)?) {
        return ax_err!(PermissionDenied);
    }

    let node = lookup_symbolic(dir, path, false)?;
    let attr = node.get_attr()?;
    if !attr.is_dir() {
        ax_err!(NotADirectory)
    } else if !attr.perm().owner_writable() {
        ax_err!(PermissionDenied)
    } else {
        let (parent, child_name) = lookup_parent(dir, path)?;
        parent.remove(&child_name, recursive)?;
        Ok(())
    }
}

pub(crate) fn current_dir() -> AxResult<String> {
    Ok(CURRENT_DIR_PATH.lock().clone())
}

pub(crate) fn set_current_dir(path: &str) -> AxResult {
    let mut abs_path = absolute_path(path)?;
    if !abs_path.ends_with('/') {
        abs_path += "/";
    }
    if abs_path == "/" {
        *CURRENT_DIR.lock() = ROOT_DIR.clone();
        *CURRENT_DIR_PATH.lock() = "/".into();
        return Ok(());
    }

    let node = lookup(None, &abs_path)?;
    let attr = node.get_attr()?;
    if !attr.is_dir() {
        ax_err!(NotADirectory)
    } else if !attr.perm().owner_executable() {
        ax_err!(PermissionDenied)
    } else {
        *CURRENT_DIR.lock() = node;
        *CURRENT_DIR_PATH.lock() = abs_path;
        Ok(())
    }
}

pub(crate) fn link(dir: Option<&VfsNodeRef>, path: &str, target_path: &str) -> AxResult {
    debug!("link {} to {}", path, target_path);
    if path.is_empty() {
        return ax_err!(NotFound);
    } else if path.ends_with('/') {
        return ax_err!(NotADirectory);
    }

    let target = lookup_symbolic(dir, target_path, false)?;
    let handle = target.get_link_handle()?;
    debug!("after get target");

    let parent = parent_node_of(dir, path);
    let (ppath, name) = axfs_vfs::path::split_parent_name(path);
    debug!("ppath = {:?}, name = {}", &ppath, &name);

    let dp = if let Some(p) = ppath {
        lookup_symbolic(Some(&parent), p.as_str(), true)?
    } else {
        parent.clone()
    };

    dp.link(name.as_str(), &handle)
}

pub(crate) fn symblink(dir: Option<&VfsNodeRef>, path: &str, target_path: &str) -> AxResult {
    debug!("symblink {} to {}", path, target_path);
    if path.is_empty() {
        return ax_err!(NotFound);
    } else if path.ends_with('/') {
        return ax_err!(NotADirectory);
    }

    lookup_symbolic(dir, target_path, false)?;

    let parent = parent_node_of(dir, path);
    let (ppath, name) = axfs_vfs::path::split_parent_name(path);

    let dp = if let Some(p) = ppath {
        lookup_symbolic(Some(&parent), p.as_str(), true)?
    } else {
        parent.clone()
    };

    dp.symlink(name.as_str(), target_path)
}

pub(crate) fn lookup_symbolic(
    dir: Option<&VfsNodeRef>,
    path: &str,
    final_jump: bool,
) -> AxResult<VfsNodeRef> {
    let mut count: usize = 0;
    _lookup_symbolic(dir, path, &mut count, 20, final_jump, false)
}

pub(crate) fn lookup_parent(
    dir: Option<&VfsNodeRef>,
    path: &str,
) -> AxResult<(VfsNodeRef, String)> {
    let mut count: usize = 0;
    let names = axfs_vfs::path::split_path(path);
    Ok((
        _lookup_symbolic(dir, path, &mut count, 20, false, true)?,
        names[names.len() - 1].clone(),
    ))
}

fn _lookup_symbolic(
    dir: Option<&VfsNodeRef>,
    path: &str,
    count: &mut usize,
    max_count: usize,
    final_jump: bool,
    return_parent: bool,
) -> AxResult<VfsNodeRef> {
    debug!("_lookup_symbolic({}, {})", path, count);
    if path.is_empty() {
        return ax_err!(NotFound);
    }
    let parent = parent_node_of(dir, path);
    let is_dir = path.ends_with('/');
    let path = path.trim_matches('/');
    let names = axfs_vfs::path::split_path(path);

    let mut cur = parent.clone();

    if names.len() <= 1 && return_parent {
        return Ok(cur);
    }

    for (idx, name) in names.iter().enumerate() {
        if idx == names.len() - 1 && return_parent {
            return Ok(cur);
        }
        let vnode = cur.clone().lookup(name.as_str())?;
        let ty = vnode.get_attr()?.file_type();
        if ty == VfsNodeType::SymLink {
            if idx == names.len() - 1 && !final_jump {
                return Ok(vnode);
            }
            *count += 1;
            if *count > max_count {
                return Err(VfsError::NotFound);
            }
            let mut new_path = vnode.get_path()?;
            let rest_path = names[idx + 1..].join("/");
            if !rest_path.is_empty() {
                new_path += "/";
                new_path += &rest_path;
            }
            if is_dir {
                new_path += "/";
            }
            debug!("follow {}", path);
            return _lookup_symbolic(None, &new_path, count, max_count, final_jump, return_parent);
        } else if idx == names.len() - 1 {
            if is_dir && !ty.is_dir() {
                return Err(AxError::NotADirectory);
            }
            return Ok(vnode);
        } else {
            match ty {
                VfsNodeType::Dir => {
                    cur = vnode.clone();
                }
                VfsNodeType::File => {
                    return Err(AxError::NotADirectory);
                }
                _ => panic!("unsupport type"),
            };
        }
    }

    panic!("_lookup_symbolic");
}

/// Close filesystem, should be called when shutting down.
pub fn close_main_fs() {
    debug!("close main fs");
    ROOT_DIR.close()
}

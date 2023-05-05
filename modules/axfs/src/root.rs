//! Root directory of the filesystem
//!
//! TODO: it doesn't work very well if the mount points have containment relationships.
//! 文件系统的根目录。
//!
//! 它将主文件系统和挂载的文件系统组织在一起。主文件系统通常是物理存储介质(如FAT)。
//! 挂载的文件系统可以是各种类型,如devfs、procfs等。
//!
//! # 实现
//!
//! 根目录包含:
//!
//! - `main_fs`: 主文件系统
//! - `mounts`: 挂载的数据结构,包含挂载点路径和文件系统
//!
//! 大多数操作都委托给 `main_fs` 或 `mounts` 中对应的文件系统。但创建/删除挂载点目录
//! 以及挂载/取消挂载文件系统的操作由根目录自己处理。
//!
//! # 用法
//!
//! 可以通过 `init_rootfs()` 初始化根目录。 然后使用各种方法在根目录下查找、创建和删除
//! 文件和目录。可以通过 `set_current_dir()` 改变当前工作目录。
use alloc::{string::String, sync::Arc, vec::Vec};
use axerrno::{ax_err, AxError, AxResult};
use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType, VfsOps, VfsResult};
use axsync::Mutex;
use lazy_init::LazyInit;

use crate::{api::FileType, fs};

static CURRENT_DIR_PATH: Mutex<String> = Mutex::new(String::new());
static CURRENT_DIR: LazyInit<Mutex<VfsNodeRef>> = LazyInit::new();

#[cfg(feature = "fatfs")]
type MainFileSystem = fs::fatfs::FatFileSystem;

/// 表示一个挂载点的数据结构
struct MountPoint {
    path: &'static str,
    fs: Arc<dyn VfsOps>,
}

/// 文件系统的根目录
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
    /// 创建一个新的根目录
    pub const fn new(main_fs: Arc<dyn VfsOps>) -> Self {
        Self {
            main_fs,
            mounts: Vec::new(),
        }
    }
    /// 在路径`path`下挂载文件系统`fs`
    pub fn mount(&mut self, path: &'static str, fs: Arc<dyn VfsOps>) -> AxResult {
        if path == "/" {
            return ax_err!(InvalidInput, "cannot mount root filesystem");
        }
        if !path.starts_with('/') {
            return ax_err!(InvalidInput, "mount path must start with '/'");
        }
        if self.mounts.iter().any(|mp| mp.path == path) {       // 检查是否已经挂载
            return ax_err!(InvalidInput, "mount point already exists");
        }
        // create the mount point in the main filesystem if it does not exist
        MAIN_FS.root_dir().create(path, FileType::Dir)?;
        fs.mount(path, MAIN_FS.root_dir().lookup(path)?)?;
        self.mounts.push(MountPoint::new(path, fs));
        Ok(())
    }
    /// 取消挂载路径`path`下的文件系统
    pub fn _umount(&mut self, path: &str) {
        self.mounts.retain(|mp| mp.path != path);
    }
    /// 检查路径`path`是否已经挂载
    pub fn contains(&self, path: &str) -> bool {
        self.mounts.iter().any(|mp| mp.path == path)
    }
    /// 在路径`path`下查找已经挂载文件系统
    fn lookup_mounted_fs<F, T>(&self, path: &str, f: F) -> AxResult<T>
        where
            F: FnOnce(Arc<dyn VfsOps>, &str) -> AxResult<T>,
    {
        debug!("lookup at root: {}", path);
        let path = path.trim_matches('/');      // 去掉开头的'/'和结尾的'/'
        if let Some(rest) = path.strip_prefix("./") {   // 如果是以'./'开头,则去掉
            return self.lookup_mounted_fs(rest, f);            //递归调用，把形如'././././xxx'的路径转换为'xxx'
        }
        /*
        这里为什么可以去掉开头的'/'呢?
        猜测：因为`lookup_mounted_fs()`是根目录的方法，是在根目录上调用的，所以`path`一定是以'/'开头的，并且相对路径和绝对路径只差一个'/'。
         */

        let mut idx = 0;
        let mut max_len = 0;

        // Find the filesystem that has the longest mounted path match
        // TODO: more efficient, e.g. trie
        for (i, mp) in self.mounts.iter().enumerate() {
            // skip the first '/'
            if path.starts_with(&mp.path[1..]) && mp.path.len() - 1 > max_len { // 如果`path`以已有的挂载点路径开头
                max_len = mp.path.len() - 1;
                idx = i;
            }
        }

        if max_len == 0 {
            f(self.main_fs.clone(), path) // not matched any mount point // 没有匹配到挂载点，直接在主文件系统上继续
        } else {
            f(self.mounts[idx].fs.clone(), &path[max_len..]) // matched at `idx`  // 在路径匹配的挂载点上继续
        }
    }
}

impl VfsNodeOps for RootDirectory {
    axfs_vfs::impl_vfs_dir_default! {}

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        self.main_fs.root_dir().get_attr()          // 获取属性时,直接委托给主文件系统的根目录
    }

    fn lookup(self: Arc<Self>, path: &str) -> VfsResult<VfsNodeRef> {
        self.lookup_mounted_fs(path, |fs, rest_path| fs.root_dir().lookup(rest_path))       // 这个闭包作用在lookup_mounted_fs()的最后几行，看起来是在挂载点上查找
    }

    /// 创建文件时,先找到匹配的挂载点,然后在对应文件系统创建,或者如果已存在则直接返回OK。
    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult {
        self.lookup_mounted_fs(path, |fs, rest_path| {
            if rest_path.is_empty() {
                Ok(()) // already exists
            } else {
                fs.root_dir().create(rest_path, ty)
            }
        })
    }
    /// 删除文件时,同样先找到挂载点,但不能删除挂载点本身,只能在对应文件系统删除,否则返回权限错误。
    fn remove(&self, path: &str) -> VfsResult {
        self.lookup_mounted_fs(path, |fs, rest_path| {
            if rest_path.is_empty() {
                ax_err!(PermissionDenied) // cannot remove mount points
            } else {
                fs.root_dir().remove(rest_path)
            }
        })
    }
}
/// 初始化根文件系统。
///
/// 这个函数会做以下工作:
/// 1. 根据配置选择主文件系统,如fatfs。
/// 2. 初始化一个`RootDirectory`作为根目录。
/// 3. 如果启用了"devfs"特征,则会:
///     - 创建一个`DeviceFileSystem`作为device文件系统
///     - 在其中添加null、zero和bar三个设备
///     - 将其挂载到根目录的/dev下
/// 4. 初始化全局的`ROOT_DIR`为根目录。
/// 5. 初始化全局的`CURRENT_DIR`为根目录。
/// 6. 设置全局的当前目录路径为"/"。
///
/// 所以,这个函数会初始化文件系统的根目录,并在上面挂载必要的其它文件系统,为整个文件系统的使用做好准备。
/// 之后,用户可以通过`ROOT_DIR`来访问根目录,通过`CURRENT_DIR`来访问当前目录。
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
        let devfs = fs::devfs::DeviceFileSystem::new();
        let foo_dir = devfs.mkdir("foo");
        devfs.add("null", Arc::new(null));
        devfs.add("zero", Arc::new(zero));
        foo_dir.add("bar", Arc::new(bar));

        root_dir
            .mount("/dev", Arc::new(devfs))
            .expect("failed to mount devfs at /dev");
    }

    ROOT_DIR.init_by(Arc::new(root_dir));
    CURRENT_DIR.init_by(Mutex::new(ROOT_DIR.clone()));
    *CURRENT_DIR_PATH.lock() = "/".into();
}

/// 获取路径`path`的父目录节点。
///
/// - dir: 可选的目录节点引用,代表路径的起始目录
/// - path: 要解析的路径
/// - 返回值: VfsNodeRef,代表路径的父目录节点
///
/// 如果路径`path`是绝对路径,则返回根目录节点。
/// 否则返回当前目录节点,或如果给定了`dir`选项,则返回`dir`对应的节点。
///
/// 这个函数的主要作用是在路径解析的过程中,得到一个路径的父目录节点,以继续递归解析路径。
fn parent_node_of(dir: Option<&VfsNodeRef>, path: &str) -> VfsNodeRef {
    if path.starts_with('/') {
        ROOT_DIR.clone()
    } else {
        dir.cloned().unwrap_or_else(|| CURRENT_DIR.lock().clone())
        /*
        如果`dir`是`None`,则调用 unwrap_or_else的回调函数。在回调函数中,我们获取`CURRENT_DIR`的互斥锁,并克隆其值返回。
        所以,总体逻辑是:
            优先使用传入的目录`dir`,如果不存在则 fallback 到当前目录 CURRENT_DIR。
        等价于
        let dir = match dir {
            Some(d) => d.clone(),
            None => CURRENT_DIR.lock().clone()
        };
         */
    }
}
/// 将路径`path`转换为绝对路径。
/// 如果`path`是绝对路径,则直接返回。
///
/// 似乎没有对'..'和'.'进行处理
pub(crate) fn absolute_path(path: &str) -> AxResult<String> {
    if path.starts_with('/') {
        Ok(axfs_vfs::path::canonicalize(path))
    } else {
        let path = CURRENT_DIR_PATH.lock().clone() + path;
        Ok(axfs_vfs::path::canonicalize(&path))
    }
}
/// 在目录`dir`下查找路径`path`对应的节点。
///
/// 这个函数会递归地解析路径`path`,获取路径上的每个目录节点并进入,直到找到目标节点。
///
/// 如果路径`path`以`/`结束,且找到的节点不是目录,则返回错误`NotADirectory`。
/// 否则返回找到的节点。
///
/// 如果路径`path`是空的,返回`NotFound`错误。
///
/// 用于通过路径来查找文件系统上的节点,是文件系统调用的基础。
pub(crate) fn lookup(dir: Option<&VfsNodeRef>, path: &str) -> AxResult<VfsNodeRef> {
    // 如果路径是空的,返回`NotFound`错误
    if path.is_empty() {
        return ax_err!(NotFound);
    }
    // 获取路径的父目录节点,并在其上继续查找
    let node = parent_node_of(dir, path).lookup(path)?;
    // 如果路径以`/`结束,且找到的节点不是目录,返回`NotADirectory`错误
    if path.ends_with('/') && !node.get_attr()?.is_dir() {
        ax_err!(NotADirectory)
    } else {
        Ok(node)
    }
}
/// 在目录`dir`下创建文件节点`path`。
///
/// 这个函数会获取路径`path`的父目录节点,并在其上调用`create()`创建文件节点。
/// 然后返回创建好的文件节点。
///
/// 如果路径`path`是空的或以`/`结束,会返回错误。空路径或以`/`结束的路径不能表示文件。
///
/// 用于在文件系统中创建文件节点,是文件创建操作的实现。
pub(crate) fn create_file(dir: Option<&VfsNodeRef>, path: &str) -> AxResult<VfsNodeRef> {
    // 如果路径是空的,返回`NotFound`错误
    if path.is_empty() {
        return ax_err!(NotFound);
    } else if path.ends_with('/') {
        return ax_err!(NotADirectory);
    }
    // 获取路径的父目录节点
    let parent = parent_node_of(dir, path);
    // 在父目录节点上创建文件节点
    parent.create(path, VfsNodeType::File)?;
    // 返回创建好的文件节点
    parent.lookup(path)
}

/// 在目录`dir`下创建目录节点`path`。
///
/// 这个函数会先检查路径`path`是否已经存在。如果存在,返回`AlreadyExists`错误。
/// 否则获取路径`path`的父目录节点,并在其上创建目录节点。
///
/// 用于在文件系统中创建目录节点。它避免了重复创建已存在的目录节点。
pub(crate) fn create_dir(dir: Option<&VfsNodeRef>, path: &str) -> AxResult {
    match lookup(dir, path) {
        Ok(_) => ax_err!(AlreadyExists),
        Err(AxError::NotFound) => parent_node_of(dir, path).create(path, VfsNodeType::Dir),
        Err(e) => Err(e),
    }
    // 感觉好巧妙
}
/// 删除文件节点`path`。
///
/// 这个函数会首先查找路径`path`对应的节点。如果节点是目录,返回`IsADirectory`错误。
/// 如果节点不是文件,或者文件没有写权限,返回`PermissionDenied`错误。
/// 否则获取父目录节点并删除文件节点。
///
/// 用于在文件系统中删除文件节点。它会先判断节点类型和权限,确保我们删除的是可以删除的文件节点。
pub(crate) fn remove_file(dir: Option<&VfsNodeRef>, path: &str) -> AxResult {
    let node = lookup(dir, path)?;
    let attr = node.get_attr()?;
    if attr.is_dir() {
        ax_err!(IsADirectory)
    }
    // 如果文件无写权限,返回'PermissionDenied'错误
    else if !attr.perm().owner_writable() {
        ax_err!(PermissionDenied)
    } else {
        parent_node_of(dir, path).remove(path)
    }
}
/// 删除目录节点`path`。
///
/// 这个函数会先判断路径`path`的有效性。如果路径为空、以`/`开头、等于`.`、等于`..`
/// 或以`/.`或`/..`结束,则返回错误。如果路径包含根目录,也返回错误。
///
/// 然后,函数查找路径`path`对应的节点。如果不是目录或无删除权限,返回错误。
/// 否则获取父目录节点并删除目录节点。
///
/// 用于在文件系统中删除目录节点。它会首先判断路径和节点的有效性与权限,确保我们
/// 删除的是一个可删除的目录节点。
pub(crate) fn remove_dir(dir: Option<&VfsNodeRef>, path: &str) -> AxResult {
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
    // 如果路径包含(某挂载点的)根目录,返回`PermissionDenied`错误
    if ROOT_DIR.contains(&absolute_path(path)?) {
        return ax_err!(PermissionDenied);
    }

    let node = lookup(dir, path)?;
    let attr = node.get_attr()?;
    if !attr.is_dir() {
        ax_err!(NotADirectory)
    } else if !attr.perm().owner_writable() {
        ax_err!(PermissionDenied)
    } else {
        parent_node_of(dir, path).remove(path)
    }
}
/// 返回当前目录。
pub(crate) fn current_dir() -> AxResult<String> {
    Ok(CURRENT_DIR_PATH.lock().clone())
}
/// 设置当前工作目录为`path`。
///
/// 这个函数会首先获取路径`path`的绝对路径。如果路径不以`/`结束,在其后追加`/`。
/// 如果绝对路径是`/`,设置当前目录为根目录。
/// 否则,查找绝对路径对应的节点。如果不是目录或无执行权限,返回错误。
/// 否则设置当前目录为该节点,并保存当前目录路径。
///
/// 用于在文件系统中设置进程的当前工作目录。它会判断目录节点的有效性与权限, 确保设置的是一个可进入的目录。
pub(crate) fn set_current_dir(path: &str) -> AxResult {
    let mut abs_path = absolute_path(path)?;
    if !abs_path.ends_with('/') {
        abs_path += "/";
    }
    // 如果绝对路径是`/`,设置当前目录为根目录
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


/*
//尝试用字典树取代vector，上面的TODO内容
struct PathTrie {
    children: BTreeMap<char, PathTrie>,
    mount_point: Option<MountPoint>,
}

impl PathTrie {
    fn insert(&mut self, path: &str, mp: MountPoint) {
        let mut node = self;
        for c in path.chars() {
            if let Some(child) = node.children.get_mut(&c) {
                node = child;
            } else {
                let child = PathTrie::default();
                node.children.insert(c, child);
                node = &mut child;
            }
        }
        node.mount_point = Some(mp);
    }

    fn longest_prefix(&self, path: &str) -> Option<(usize, &MountPoint)> {
        let mut node = self;
        let mut len = 0;
        for c in path.chars() {
            if let Some(child) = node.children.get(&c) {
                node = child;
                len += 1;
                if node.mount_point.is_some() {
                    return Some((len, node.mount_point.as_ref().unwrap()));
                }
            } else {
                break;
            }
        }
        None
    }
}
 */
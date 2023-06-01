use alloc::string::String;
use alloc::sync::Arc;
use core::cell::{RefCell, UnsafeCell};
use core::ptr::NonNull;

use axfs_vfs::{VfsDirEntry, VfsError, VfsNodePerm, VfsResult};
use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType, VfsOps};
use ext2fs::{self, ext2err::Ext2Error, BlockDevice, BLOCK_SIZE};

use crate::dev::Disk;

pub struct DiskAdapter {
    inner: RefCell<Disk>,
}

impl BlockDevice for DiskAdapter {
    fn block_num(&self) -> usize {
        self.inner.borrow_mut().size() as usize / BLOCK_SIZE
    }
    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        assert!(buf.len() == BLOCK_SIZE);
        let mut inner = self.inner.borrow_mut();
        let true_block_size = inner.true_block_size();
        let num_block = BLOCK_SIZE / true_block_size;

        for i in 0..num_block {
            let pos = block_id * BLOCK_SIZE + i * true_block_size;
            inner.set_position(pos as _);
            let res = inner.read_one(&mut buf[i * true_block_size..(i + 1) * true_block_size]);
            assert_eq!(res.unwrap(), true_block_size);
        }
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        assert!(buf.len() == BLOCK_SIZE);
        let mut inner = self.inner.borrow_mut();
        let true_block_size = inner.true_block_size();
        let num_block = BLOCK_SIZE / true_block_size;

        for i in 0..num_block {
            let pos = block_id * BLOCK_SIZE + i * true_block_size;
            inner.set_position(pos as _);
            let res = inner.write_one(&buf[i * true_block_size..(i + 1) * true_block_size]);
            assert_eq!(res.unwrap(), true_block_size);
        }
    }
}

pub struct Ext2DirWrapper(ext2fs::Inode, NonNull<Ext2FileSystem>);
pub struct Ext2FileWrapper(ext2fs::Inode, NonNull<Ext2FileSystem>);
pub struct Ext2SymlinkWrapper(ext2fs::Inode, NonNull<Ext2FileSystem>);

pub struct Ext2FileSystem {
    inner: Arc<ext2fs::Ext2FileSystem>,
    root_dir: UnsafeCell<Option<VfsNodeRef>>,
}
unsafe impl Send for Ext2DirWrapper {}
unsafe impl Sync for Ext2DirWrapper {}
unsafe impl Send for Ext2FileWrapper {}
unsafe impl Sync for Ext2FileWrapper {}
unsafe impl Send for Ext2SymlinkWrapper {}
unsafe impl Sync for Ext2SymlinkWrapper {}
unsafe impl Send for Ext2FileSystem {}
unsafe impl Sync for Ext2FileSystem {}

impl Ext2FileSystem {
    pub fn new(disk: Disk) -> Self {
        let block_device = Arc::new(DiskAdapter {
            inner: RefCell::new(disk),
        });
        let timer = Arc::new(ext2fs::timer::ZeroTimeProvider);
        let inner = ext2fs::Ext2FileSystem::open(block_device, timer);
        Self {
            inner,
            root_dir: UnsafeCell::new(None),
        }
    }

    fn new_file(inode: ext2fs::Inode, fs_ptr: NonNull<Ext2FileSystem>) -> Arc<Ext2FileWrapper> {
        assert!(inode.is_file());
        Arc::new(Ext2FileWrapper(inode, fs_ptr))
    }

    fn new_dir(inode: ext2fs::Inode, fs_ptr: NonNull<Ext2FileSystem>) -> Arc<Ext2DirWrapper> {
        assert!(inode.is_dir());
        Arc::new(Ext2DirWrapper(inode, fs_ptr))
    }

    fn new_sym(inode: ext2fs::Inode, fs_ptr: NonNull<Ext2FileSystem>) -> Arc<Ext2SymlinkWrapper> {
        assert!(inode.is_symlink());
        Arc::new(Ext2SymlinkWrapper(inode, fs_ptr))
    }

    pub fn init(&'static self) {
        let root_inode = ext2fs::Ext2FileSystem::root_inode(&self.inner);
        let fs_ptr: NonNull<Ext2FileSystem> =
            NonNull::new(self as *const _ as *mut Ext2FileSystem).unwrap();
        unsafe { *self.root_dir.get() = Some(Self::new_dir(root_inode, fs_ptr)) }
    }
}

impl VfsOps for Ext2FileSystem {
    fn root_dir(&self) -> VfsNodeRef {
        let root_dir = unsafe { (*self.root_dir.get()).as_ref().unwrap() };
        root_dir.clone()
    }

    fn umount(&self) -> VfsResult {
        debug!("ext2fs unmount");
        self.inner.close();
        Ok(())
    }
}

impl VfsNodeOps for Ext2SymlinkWrapper {
    axfs_vfs::impl_ext2_common! {}

    axfs_vfs::impl_ext2_linkable! {}

    fn get_path(&self) -> VfsResult<String> {
        self.0.path_name().map_err(map_ext2_err)
    }
}

impl VfsNodeOps for Ext2FileWrapper {
    axfs_vfs::impl_vfs_non_dir_default! {}

    axfs_vfs::impl_ext2_common! {}

    axfs_vfs::impl_ext2_linkable! {}

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        self.0.read_at(offset as _, buf).map_err(map_ext2_err)
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let res = self.0.write_at(offset as _, buf).map_err(map_ext2_err);
        let _ = self.0.flush();
        res
    }

    fn truncate(&self, size: u64) -> VfsResult {
        let res = self.0.ftruncate(size as _).map_err(map_ext2_err);
        let _ = self.0.flush();
        res
    }
}

impl VfsNodeOps for Ext2DirWrapper {
    axfs_vfs::impl_vfs_dir_default! {}

    axfs_vfs::impl_ext2_common! {}

    fn parent(&self) -> Option<VfsNodeRef> {
        if let Some(p) = self.0.find("..").ok() {
            Some(Ext2FileSystem::new_dir(p, self.1))
        } else {
            None
        }
    }

    fn lookup(self: Arc<Self>, path: &str) -> VfsResult<VfsNodeRef> {
        debug!("lookup at ext2fs: {}", path);
        let path = path.trim_matches('/');
        if path.is_empty() || path == "." {
            return Ok(self.clone());
        }
        if let Some(rest) = path.strip_prefix("./") {
            return self.lookup(rest);
        }

        let (parent, name) = self.0.lookup_parent(path).map_err(map_ext2_err)?;
        let inode = parent.find(name.as_str()).map_err(map_ext2_err)?;

        if inode.is_dir() {
            return Ok(Ext2FileSystem::new_dir(inode, self.1));
        } else if inode.is_file() {
            return Ok(Ext2FileSystem::new_file(inode, self.1));
        } else if inode.is_symlink() {
            return Ok(Ext2FileSystem::new_sym(inode, self.1));
        } else {
            return Err(axerrno::AxError::Unsupported);
        }
    }

    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult {
        debug!("create {:?} at ext2fs: {}", ty, path);
        let path = path.trim_matches('/');
        if path.is_empty() || path == "." {
            return Ok(());
        }
        if let Some(rest) = path.strip_prefix("./") {
            return self.create(rest, ty);
        }

        match ty {
            VfsNodeType::Dir => (),
            VfsNodeType::File => (),
            _ => return Err(VfsError::Unsupported),
        }

        let (parent, name) = self.0.lookup_parent(path).map_err(map_ext2_err)?;

        let res = match ty {
            VfsNodeType::Dir => parent
                .create_dir(name.as_str())
                .map(|_| ())
                .map_err(map_ext2_err),
            VfsNodeType::File => parent
                .create_file(name.as_str())
                .map(|_| ())
                .map_err(map_ext2_err),
            _ => panic!("unsupport type"),
        };
        let _ = self.0.flush();
        res
    }

    fn remove(&self, path: &str, recursive: bool) -> VfsResult {
        debug!("remove at ext2fs: {}", path);
        let path = path.trim_matches('/');
        assert!(!path.is_empty()); // already check at `root.rs`
        if let Some(rest) = path.strip_prefix("./") {
            return self.remove(rest, recursive);
        }

        let (parent, name) = self.0.lookup_parent(path).map_err(map_ext2_err)?;

        let inode = parent.find(name.as_str()).map_err(map_ext2_err)?;
        if inode.is_dir() {
            parent
                .rm_dir(name.as_str(), recursive)
                .map_err(map_ext2_err)?;
            let _ = self.0.flush();
            return Ok(());
        } else if inode.is_file() {
            parent.rm_file(name.as_str()).map_err(map_ext2_err)?;
            let _ = self.0.flush();
            return Ok(());
        } else if inode.is_symlink() {
            parent.rm_symlink(name.as_str()).map_err(map_ext2_err)?;
            let _ = self.0.flush();
            return Ok(());
        } else {
            panic!("Unsuport type");
        };
    }

    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        debug!("read_dir ext2 at {}", start_idx);
        let dir_entries = self.0.read_dir().map_err(map_ext2_err)?;
        let mut iter = dir_entries.into_iter().skip(2).skip(start_idx);
        for (i, out_entry) in dirents.iter_mut().enumerate() {
            let x = iter.next();
            match x {
                Some((name, direntry)) => {
                    let (ty, _) = map_imode(ext2fs::layout::DiskInode::_file_code_to_disk(
                        direntry.file_type,
                    ));
                    *out_entry = VfsDirEntry::new(name.as_str(), ty);
                }
                _ => {
                    return Ok(i);
                }
            }
        }
        unreachable!();
    }

    fn link(&self, name: &str, handle: &axfs_vfs::LinkHandle) -> VfsResult {
        debug!("ext2 link {} to inode {}", name, handle.inode_id);
        if self.1.as_ptr() as usize != handle.fssp_ptr {
            return Err(VfsError::InvalidInput);
        }
        let res = self.0.link(name, handle.inode_id).map_err(map_ext2_err);
        let _ = self.0.flush();
        res
    }

    fn symlink(&self, name: &str, path: &str) -> VfsResult {
        debug!("ext2 symbolic link {} to {}", name, path);
        if !path.starts_with("/") {
            return Err(VfsError::InvalidInput);
        }
        let res = self.0.symlink(name, path).map_err(map_ext2_err);
        let _ = self.0.flush();
        res
    }
}

// fn split_path(path: &str) -> Vec<String> {
//     let mut names: Vec<String> = Vec::new();

//     for name in path.split("/") {
//         names.push(String::from(name))
//     }
//     names
// }

fn map_imode(imode: u16) -> (VfsNodeType, VfsNodePerm) {
    // debug!("map_imode {} {}", imode & 0xf000, imode & 0xfff);
    let type_code = imode & 0xf000;
    let ty = match type_code {
        ext2fs::layout::EXT2_S_IFREG => VfsNodeType::File,
        ext2fs::layout::EXT2_S_IFDIR => VfsNodeType::Dir,
        ext2fs::layout::EXT2_S_IFCHR => VfsNodeType::CharDevice,
        ext2fs::layout::EXT2_S_IFBLK => VfsNodeType::BlockDevice,
        ext2fs::layout::EXT2_S_IFIFO => VfsNodeType::Fifo,
        ext2fs::layout::EXT2_S_IFSOCK => VfsNodeType::Socket,
        ext2fs::layout::EXT2_S_IFLNK => VfsNodeType::SymLink,
        _ => panic!("Unsupport type"),
    };
    let perm = ext2fs::layout::IMODE::from_bits_truncate(imode);
    // debug!("origin perm {}", perm.bits());
    let mut vfs_perm = VfsNodePerm::from_bits_truncate(0);

    if perm.contains(ext2fs::layout::IMODE::EXT2_S_IXOTH) {
        vfs_perm |= VfsNodePerm::OTHER_EXEC;
    }
    if perm.contains(ext2fs::layout::IMODE::EXT2_S_IWOTH) {
        vfs_perm |= VfsNodePerm::OTHER_WRITE;
    }
    if perm.contains(ext2fs::layout::IMODE::EXT2_S_IROTH) {
        vfs_perm |= VfsNodePerm::OTHER_READ;
    }

    if perm.contains(ext2fs::layout::IMODE::EXT2_S_IXGRP) {
        vfs_perm |= VfsNodePerm::GROUP_EXEC;
    }
    if perm.contains(ext2fs::layout::IMODE::EXT2_S_IWGRP) {
        vfs_perm |= VfsNodePerm::GROUP_WRITE;
    }
    if perm.contains(ext2fs::layout::IMODE::EXT2_S_IRGRP) {
        vfs_perm |= VfsNodePerm::GROUP_READ;
    }

    if perm.contains(ext2fs::layout::IMODE::EXT2_S_IXUSR) {
        vfs_perm |= VfsNodePerm::OWNER_EXEC;
    }
    if perm.contains(ext2fs::layout::IMODE::EXT2_S_IWUSR) {
        vfs_perm |= VfsNodePerm::OWNER_WRITE;
    }
    if perm.contains(ext2fs::layout::IMODE::EXT2_S_IRUSR) {
        vfs_perm |= VfsNodePerm::OWNER_READ;
    }

    // debug!("final perm {}", vfs_perm.bits());

    (ty, vfs_perm)
}

const fn map_ext2_err(err: Ext2Error) -> VfsError {
    use Ext2Error::*;
    match err {
        AlreadyExists => VfsError::AlreadyExists,
        DirectoryIsNotEmpty => VfsError::DirectoryNotEmpty,
        NotFound => VfsError::NotFound,
        NotEnoughSpace => VfsError::StorageFull,
        NotADir => VfsError::NotADirectory,
        NotAFile => VfsError::IsADirectory,
        InvalidResource => VfsError::NotFound,
        _ => VfsError::InvalidInput,
    }
}

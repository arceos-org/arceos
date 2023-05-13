use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::string::String;
use core::cell::{RefCell, UnsafeCell};

use axfs_vfs::{VfsDirEntry, VfsError, VfsNodePerm, VfsResult};
use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType, VfsOps};
use ext2fs::{self, BlockDevice, BLOCK_SIZE};

use crate::dev::Disk;

pub struct DiskAdapter {
    inner: RefCell<Disk>
}

impl BlockDevice for DiskAdapter {
    fn block_num(&self) -> usize {
        self.inner.borrow_mut().size() as usize/BLOCK_SIZE
    }
    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        assert!(buf.len() == BLOCK_SIZE);
        let mut inner = self.inner.borrow_mut();
        let true_block_size = inner.true_block_size();
        let num_block = BLOCK_SIZE/true_block_size;

        for i in 0..num_block {
            let pos = block_id * BLOCK_SIZE + i * true_block_size;
            inner.set_position(pos as _);
            let res = inner.read_one(&mut buf[i*true_block_size..(i+1)*true_block_size]);
            assert_eq!(res.unwrap(), true_block_size);
        }
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        assert!(buf.len() == BLOCK_SIZE);
        let mut inner = self.inner.borrow_mut();
        let true_block_size = inner.true_block_size();
        let num_block = BLOCK_SIZE/true_block_size;

        for i in 0..num_block {
            let pos = block_id * BLOCK_SIZE + i * true_block_size;
            inner.set_position(pos as _);
            let res = inner.write_one(&buf[i*true_block_size..(i+1)*true_block_size]);
            assert_eq!(res.unwrap(), true_block_size);
        }
    }
}

pub struct Ext2DirWrapper(ext2fs::Inode);
pub struct Ext2FileWrapper(ext2fs::Inode);

pub struct Ext2FileSystem {
    inner: Arc<ext2fs::Ext2FileSystem>,
    root_dir: UnsafeCell<Option<VfsNodeRef>>
}
unsafe impl Send for Ext2DirWrapper {}
unsafe impl Sync for Ext2DirWrapper {}
unsafe impl Send for Ext2FileWrapper {}
unsafe impl Sync for Ext2FileWrapper {}
unsafe impl Send for Ext2FileSystem {}
unsafe impl Sync for Ext2FileSystem {}

impl Ext2FileSystem {
    pub fn new(disk: Disk) -> Self {
        let block_device = Arc::new(DiskAdapter {
            inner: RefCell::new(disk)
        });
        let timer = Arc::new(ext2fs::timer::ZeroTimeProvider);
        let inner = ext2fs::Ext2FileSystem::open(block_device, timer);
        Self {
            inner,
            root_dir: UnsafeCell::new(None)
        }
    }

    fn new_file(inode: ext2fs::Inode) -> Arc<Ext2FileWrapper> {
        assert!(inode.is_file());
        Arc::new(Ext2FileWrapper(inode))
    }

    fn new_dir(inode: ext2fs::Inode) -> Arc<Ext2DirWrapper> {
        assert!(inode.is_dir());
        Arc::new(Ext2DirWrapper(inode))
    }

    pub fn init(&'static self) {
        let root_inode = ext2fs::Ext2FileSystem::root_inode(&self.inner);
        unsafe { *self.root_dir.get() = Some(Self::new_dir(root_inode)) }
    }
}

impl VfsOps for Ext2FileSystem {
    fn root_dir(&self) -> VfsNodeRef {
        let root_dir = unsafe { (*self.root_dir.get()).as_ref().unwrap() };
        root_dir.clone()
    }
}

impl VfsNodeOps for Ext2FileWrapper {
    axfs_vfs::impl_vfs_non_dir_default! {}

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        if let Some(disk_inode) = self.0.disk_inode() {
            let (ty, perm) = map_imode(disk_inode.i_mode);
            return Ok(VfsNodeAttr::new(perm, ty, disk_inode.i_size as _, disk_inode.i_blocks as _));
        } else {
            return Err(VfsError::NotFound);
        }
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        if let Some(num) = self.0.read_at(offset as _, buf) {
            Ok(num)
        } else {
            Err(axerrno::AxError::NotFound)
        }
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        if let Some(num) = self.0.write_at(offset as _, buf) {
            Ok(num)
        } else {
            Err(axerrno::AxError::NotFound)
        }
    }

    fn truncate(&self, size: u64) -> VfsResult {
        if let Some(success) = self.0.ftruncate(size as _) {
            if success {
                Ok(())
            } else {
                Err(axerrno::AxError::NotFound)
            }
        } else {
            Err(axerrno::AxError::NotFound)
        }
    }
}


impl VfsNodeOps for Ext2DirWrapper {
    axfs_vfs::impl_vfs_dir_default! {}

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        if let Some(disk_inode) = self.0.disk_inode() {
            let (ty, perm) = map_imode(disk_inode.i_mode);
            return Ok(VfsNodeAttr::new(perm, ty, disk_inode.i_size as _, disk_inode.i_blocks as _));
        } else {
            return Err(VfsError::NotFound);
        }
    }

    fn parent(&self) -> Option<VfsNodeRef> {
        if let Some(p) = self.0.find("..") {
            Some(Ext2FileSystem::new_dir(p))
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
        let names = split_path(path);
        let mut cur = self.0.clone();

        for (idx, name) in names.iter().enumerate() {
            if name.len() == 0 {
                continue;
            }
            if let Some(inode) = cur.find(name) {
                if idx != names.len() - 1 {
                    if !inode.is_dir() {
                        return Err(axerrno::AxError::NotFound);
                    } else {
                        cur = inode.clone();
                    }
                } else {
                    if inode.is_dir() {
                        return Ok(Ext2FileSystem::new_dir(inode));
                    } else if inode.is_file() {
                        return Ok(Ext2FileSystem::new_file(inode));
                    } else {
                        return Err(axerrno::AxError::Unsupported);
                    }
                }
            } else {
                return Err(axerrno::AxError::NotFound);
            }
        }

        panic!("lookup");
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
            _ => return Err(VfsError::Unsupported)
        }

        let names = split_path(path);
        let mut cur = self.0.clone();

        for (idx, name) in names.iter().enumerate() {
            if idx == names.len() - 1 {
                match ty {
                    VfsNodeType::Dir => {
                        if let Some(_) = cur.create_dir(name) {
                            return Ok(());
                        } else {
                            return Err(VfsError::AlreadyExists);
                        }
                    },
                    VfsNodeType::File => {
                        if let Some(_) = cur.create_file(name) {
                            return Ok(());
                        } else {
                            return Err(VfsError::AlreadyExists);
                        }
                    }
                    _ => panic!("unsupported file type")
                }
            } else {
                if let Some(inode) = cur.find(name) {
                    if inode.is_dir() {
                        cur = inode.clone();
                    } else {
                        return Err(VfsError::NotFound);
                    }
                }
            }
        }

        panic!("create");
    }

    fn remove(&self, path: &str) -> VfsResult {
        debug!("remove at ext2fs: {}", path);
        let path = path.trim_matches('/');
        assert!(!path.is_empty()); // already check at `root.rs`
        if let Some(rest) = path.strip_prefix("./") {
            return self.remove(rest);
        }

        let names = split_path(path);
        let mut cur = self.0.clone();

        for (idx, name) in names.iter().enumerate() {
            if idx == names.len() - 1 {
                if let Some(inode) = cur.find(name) {
                    if inode.is_dir() {
                        cur.rm_dir(name.as_str(), false);
                        return Ok(());
                    } else if inode.is_file() {
                        cur.rm_file(name.as_str());
                        return Ok(());
                    }
                    panic!("Unsuport type");
                } else {
                    return Err(VfsError::NotFound);
                }
            } else {
                if let Some(inode) = cur.find(name) {
                    if inode.is_dir() {
                        cur = inode.clone();
                    } else {
                        return Err(VfsError::NotFound);
                    }
                }
            }
        }

        panic!("remove");
    }

    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        debug!("read_dir at {}", start_idx);
        if let Some(dir_entries) = self.0.read_dir() {
            let mut iter = dir_entries.into_iter().skip(2).skip(start_idx);
            for (i, out_entry) in dirents.iter_mut().enumerate() {
                let x = iter.next();
                match x {
                    Some((name, direntry)) => {
                        let (ty, _) = map_imode(ext2fs::layout::DiskInode::_file_code_to_disk(direntry.file_type));
                        *out_entry = VfsDirEntry::new(name.as_str(), ty);
                    },
                    _ => {
                        return Ok(i);
                    }
                }
            }
            unreachable!();
        } else {
            return Err(VfsError::NotFound);
        }
    }
}

fn split_path(path: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();

    for name in path.split("/") {
        names.push(String::from(name))
    }
    names
}

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
        _ => panic!("Unsupport type")
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
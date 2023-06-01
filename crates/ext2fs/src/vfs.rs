use core::cmp::Ordering;
use core::mem::size_of;
use log::*;

use crate::mutex::SpinMutex;

use super::{
    config::MAX_PATH_NAME,
    layout::{
        DirEntryHead, DEFAULT_IMODE, EXT2_FT_DIR, EXT2_FT_REG_FILE, EXT2_FT_SYMLINK,
        EXT2_FT_UNKNOWN, EXT2_S_IFDIR, EXT2_S_IFLNK, EXT2_S_IFREG, IMODE, MAX_NAME_LEN,
    },
    DiskInode, Ext2Error, Ext2FileSystem, Ext2Result,
};
use alloc::string::{String, ToString};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use axfs_vfs::path::split_path;

#[derive(Clone)]
/// Ext2 filesystem vfs inode
pub struct Inode {
    file_type: u8,
    inner: Arc<SpinMutex<InodeCache>>,
}

impl Inode {
    pub(crate) fn new(inner: Arc<SpinMutex<InodeCache>>) -> Inode {
        let file_type = inner.lock().file_type();
        Self { file_type, inner }
    }

    fn access(&self) -> Ext2Result<&Arc<SpinMutex<InodeCache>>> {
        if self.inner.lock().valid {
            Ok(&self.inner)
        } else {
            Err(Ext2Error::InvalidResource)
        }
    }

    // common operations

    /// flush filesystem
    pub fn flush(&self) -> Ext2Result {
        self.access()?.lock().flush();
        Ok(())
    }

    /// Get file type
    pub fn file_type(&self) -> u8 {
        self.file_type
    }

    /// Is a dir
    pub fn is_dir(&self) -> bool {
        self.file_type == EXT2_FT_DIR
    }

    /// Is a file
    pub fn is_file(&self) -> bool {
        self.file_type == EXT2_FT_REG_FILE
    }

    /// Is a symbolic link
    pub fn is_symlink(&self) -> bool {
        self.file_type == EXT2_FT_SYMLINK
    }

    /// Get inode id
    pub fn inode_id(&self) -> Ext2Result<usize> {
        Ok(self.access()?.lock().inode_id)
    }

    /// chown
    pub fn chown(&self, uid: Option<usize>, gid: Option<usize>) -> Ext2Result {
        self.access()?.lock().chown(uid, gid);
        Ok(())
    }

    /// chmod
    pub fn chmod(&self, access: IMODE) -> Ext2Result {
        self.access()?.lock().chmod(access);
        Ok(())
    }

    /// Get disk inode
    pub fn disk_inode(&self) -> Ext2Result<DiskInode> {
        Ok(self.access()?.lock().disk_inode())
    }

    // symbolic link operation

    /// Get the path name of a symbolic link
    pub fn path_name(&self) -> Ext2Result<String> {
        let lk = self.access()?.lock();
        if self.file_type != EXT2_FT_SYMLINK {
            Err(Ext2Error::NotSymlink)
        } else {
            let mut buf = [0u8; MAX_PATH_NAME];
            let path_len = lk.read_at(0, &mut buf);
            Ok(core::str::from_utf8(&buf[..path_len]).unwrap().to_string())
        }
    }

    // file operation

    /// Change file size
    pub fn ftruncate(&self, new_size: usize) -> Ext2Result {
        self.access()?.lock().ftruncate(new_size as _)
    }

    /// Read file
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> Ext2Result<usize> {
        let lk = self.access()?.lock();
        if self.file_type != EXT2_FT_REG_FILE {
            Err(Ext2Error::NotAFile)
        } else {
            Ok(lk.read_at(offset, buf))
        }
    }

    /// Write file
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> Ext2Result<usize> {
        let mut lk = self.access()?.lock();
        if self.file_type != EXT2_FT_REG_FILE {
            Err(Ext2Error::NotAFile)
        } else {
            Ok(lk.write_at(offset, buf))
        }
    }

    /// Append data
    pub fn append(&self, buf: &[u8]) -> Ext2Result<usize> {
        let mut lk = self.access()?.lock();
        if self.file_type != EXT2_FT_REG_FILE {
            Err(Ext2Error::NotAFile)
        } else {
            Ok(lk.append(buf))
        }
    }

    // dir operation

    /// Lookup the parent of `path`, and return the rest path.
    pub fn lookup_parent(&self, path: &str) -> Ext2Result<(Self, String)> {
        let mut cur = self.clone();
        let names = split_path(path);

        for (idx, name) in names.iter().enumerate() {
            if idx == names.len() - 1 {
                return Ok((cur, name.clone()));
            } else {
                let inode = cur.find(name)?;
                if inode.is_dir() {
                    cur = inode;
                } else {
                    return Err(Ext2Error::NotADir);
                }
            }
        }
        panic!("ext2 lookup");
    }

    /// Lookup a dir
    pub fn find(&self, name: &str) -> Ext2Result<Self> {
        let lk = self.access()?.lock();
        lk.find(name).map(Self::new)
    }

    /// Create a dir
    pub fn create_dir(&self, name: &str) -> Ext2Result<Self> {
        self.create(name, EXT2_S_IFDIR | DEFAULT_IMODE.bits())
    }

    /// Create a file
    pub fn create_file(&self, name: &str) -> Ext2Result<Self> {
        self.create(name, EXT2_S_IFREG | DEFAULT_IMODE.bits())
    }

    /// Create a file/dir
    pub fn create(&self, name: &str, file_type: u16) -> Ext2Result<Self> {
        let mut lk = self.access()?.lock();
        lk.create(name, file_type).map(Self::new)
    }

    /// List die
    pub fn ls(&self) -> Ext2Result<Vec<String>> {
        let lk = self.access()?.lock();
        lk.ls()
    }

    /// Read dir
    pub fn read_dir(&self) -> Ext2Result<Vec<(String, DirEntryHead)>> {
        let lk = self.access()?.lock();
        lk.ls_direntry()
    }

    /// Check if it is an empty dir
    pub fn is_empty_dir(&self) -> Ext2Result<bool> {
        let lk = self.access()?.lock();
        lk.is_empty_dir()
    }

    /// Hard link
    pub fn link(&self, name: &str, inode_id: usize) -> Ext2Result {
        let mut lk = self.access()?.lock();
        lk.link(name, inode_id)
    }

    /// Symbolic link
    pub fn symlink(&self, name: &str, path_name: &str) -> Ext2Result {
        let mut lk = self.access()?.lock();
        lk.symlink(name, path_name)
    }

    /// Remove a file
    pub fn rm_file(&self, file_name: &str) -> Ext2Result {
        let mut lk = self.access()?.lock();
        lk.unlink(file_name, EXT2_FT_REG_FILE, false)
    }

    /// Remove a directory
    pub fn rm_dir(&self, dir_name: &str, recursive: bool) -> Ext2Result {
        let mut lk = self.access()?.lock();
        lk.unlink(dir_name, EXT2_FT_DIR, recursive)
    }

    /// Remove a symbolic link
    pub fn rm_symlink(&self, dir_name: &str) -> Ext2Result {
        let mut lk = self.access()?.lock();
        lk.unlink(dir_name, EXT2_FT_SYMLINK, false)
    }
}

/// Virtual filesystem layer over easy-fs
pub struct InodeCache {
    pub inode_id: usize,
    block_id: usize,
    block_offset: usize,
    fs: Weak<Ext2FileSystem>,

    // cache part
    file_type: u8,
    size: usize,
    blocks: Vec<u32>,
    pub valid: bool,
}

impl InodeCache {
    pub(crate) fn new(
        inode_id: usize,
        block_id: usize,
        block_offset: usize,
        fs: Arc<Ext2FileSystem>,
    ) -> Self {
        let mut inode = Self {
            inode_id,
            block_id,
            block_offset,
            fs: Arc::downgrade(&fs),
            file_type: EXT2_FT_UNKNOWN,
            size: 0,
            blocks: Vec::new(),
            valid: true,
        };
        inode.read_cache();
        inode
    }

    fn get_fs(&self) -> Arc<Ext2FileSystem> {
        self.fs.upgrade().unwrap()
    }

    /// Call a function over a disk inode to read it
    fn read_disk_inode<V>(&self, f: impl FnOnce(&DiskInode) -> V) -> V {
        let inode_block = self.get_fs().manager.lock().get_block_cache(self.block_id);
        let ret = inode_block.lock().read(self.block_offset, f);
        self.get_fs().manager.lock().release_block(inode_block);
        ret
    }
    /// Call a function over a disk inode to modify it
    fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut DiskInode) -> V) -> V {
        let inode_block = self.get_fs().manager.lock().get_block_cache(self.block_id);
        let ret = inode_block.lock().modify(self.block_offset, f);
        self.get_fs().manager.lock().release_block(inode_block);
        ret
    }

    pub(crate) fn read_cache(&mut self) {
        let mut file_type: u8 = 0;
        let mut file_size: usize = 0;
        let mut blocks: Vec<u32> = Vec::new();

        self.read_disk_inode(|disk_inode| {
            file_type = disk_inode.file_code();
            file_size = disk_inode.i_size as usize;
            blocks = disk_inode.all_data_blocks(&self.get_fs().manager, false);
        });

        self.file_type = file_type;
        self.size = file_size;
        self.blocks = blocks;
    }

    pub(crate) fn flush(&self) {
        self.get_fs().close()
    }

    pub(crate) fn file_type(&self) -> u8 {
        self.file_type
    }

    pub(crate) fn disk_inode(&self) -> DiskInode {
        self.read_disk_inode(|disk_inode| *disk_inode)
    }

    /// Find inode under a disk inode by name (DirEntry, pos, prev_offset)
    fn find_inode_id(
        &self,
        name: &str,
        disk_inode: &DiskInode,
    ) -> Option<(DirEntryHead, usize, usize)> {
        // debug!("find_inode_id");
        // assert it is a directory
        assert!(disk_inode.is_dir());
        let mut buffer = [0_u8; MAX_NAME_LEN];
        let mut dir_entry_head = DirEntryHead::empty();
        let mut offset: usize = 0;
        let mut pos: usize = 0;
        let mut prev_offset: usize = 0;

        while offset + size_of::<DirEntryHead>() < disk_inode.i_size as usize {
            assert_eq!(
                disk_inode.read_at(
                    offset,
                    dir_entry_head.as_bytes_mut(),
                    &self.get_fs().manager,
                    Some(&self.blocks)
                ),
                size_of::<DirEntryHead>()
            );
            let name_len = dir_entry_head.name_len as usize;
            let name_buffer = &mut buffer[0..name_len];
            let name_offset = offset + size_of::<DirEntryHead>();
            assert_eq!(
                disk_inode.read_at(
                    name_offset,
                    name_buffer,
                    &self.get_fs().manager,
                    Some(&self.blocks)
                ),
                name_len
            );
            if name_buffer == name.as_bytes() {
                return Some((dir_entry_head, pos, prev_offset));
            }
            prev_offset = offset;
            offset += dir_entry_head.rec_len as usize;
            pos += 1;
        }

        None
    }

    fn get_inode_id(&self, name: &str) -> Option<(DirEntryHead, usize, usize)> {
        self.read_disk_inode(|disk_inode| self.find_inode_id(name, disk_inode))
    }

    pub(crate) fn find(&self, name: &str) -> Ext2Result<Arc<SpinMutex<InodeCache>>> {
        if let Some(de) = self.get_inode_id(name).map(|(de, _, _)| de) {
            Ok(Ext2FileSystem::get_inode_cache(&self.get_fs(), de.inode as _).unwrap())
        } else {
            Err(Ext2Error::NotFound)
        }
    }

    pub(crate) fn create(
        &mut self,
        name: &str,
        mut file_type: u16,
    ) -> Ext2Result<Arc<SpinMutex<InodeCache>>> {
        if self.file_type() != EXT2_FT_DIR {
            return Err(Ext2Error::NotADir);
        }
        if name.len() >= MAX_NAME_LEN {
            return Err(Ext2Error::NameTooLong);
        }
        if name.contains('/') {
            return Err(Ext2Error::InvalidName);
        }
        if self.get_inode_id(name).is_some() {
            error!("Try to create a file already exists");
            return Err(Ext2Error::AlreadyExists);
        }
        file_type &= 0xF000;
        let new_inode_id = self.get_fs().alloc_inode().unwrap();
        let (new_inode_block_id, new_inode_block_offset) =
            self.get_fs().get_disk_inode_pos(new_inode_id);
        let inode_block = self
            .get_fs()
            .manager
            .lock()
            .get_block_cache(new_inode_block_id as _);
        inode_block
            .lock()
            .modify(new_inode_block_offset, |disk_inode: &mut DiskInode| {
                *disk_inode = DiskInode::new(DEFAULT_IMODE, file_type, 0, 0);
                let cur_time = self.get_fs().timer.get_current_time();
                disk_inode.i_atime = cur_time;
                disk_inode.i_ctime = cur_time;
            });
        self.get_fs().manager.lock().release_block(inode_block);

        let new_inode =
            Ext2FileSystem::get_inode_cache(&self.get_fs(), new_inode_id as usize).unwrap();
        self.append_dir_entry(new_inode_id as usize, name, new_inode.lock().file_type());

        if file_type == EXT2_S_IFDIR {
            // new_inode.lock().link(".", new_inode_id as usize);
            // new_inode.lock().link("..", self.inode_id);
            let mut lk = new_inode.lock();
            lk.append_dir_entry(new_inode_id as usize, ".", EXT2_FT_DIR);
            lk.append_dir_entry(self.inode_id, "..", EXT2_FT_DIR);
            lk.increase_nlink(1);

            self.increase_nlink(1);
        }

        self.get_fs().write_meta();
        Ok(new_inode)
    }

    /// can only link to file
    pub(crate) fn link(&mut self, name: &str, inode_id: usize) -> Ext2Result {
        if self.file_type() != EXT2_FT_DIR {
            return Err(Ext2Error::NotADir);
        }
        debug!("link {} to {}", name, inode_id);
        if inode_id == 0 {
            return Err(Ext2Error::InvalidInodeId);
        }

        // link to self
        if self.inode_id == inode_id {
            return Err(Ext2Error::LinkToSelf);
        }

        if let Some(inode) = Ext2FileSystem::get_inode_cache(&self.get_fs(), inode_id) {
            if self.get_inode_id(name).is_some() {
                // already exists
                Err(Ext2Error::AlreadyExists)
            } else {
                let lk = inode.lock();
                if lk.file_type() != EXT2_FT_REG_FILE {
                    return Err(Ext2Error::LinkToDir);
                }
                self.append_dir_entry(inode_id, name, EXT2_FT_REG_FILE);
                lk.increase_nlink(1);
                self.get_fs().write_meta();
                Ok(())
            }
        } else {
            Err(Ext2Error::InvalidInodeId)
        }
    }

    pub(crate) fn symlink(&mut self, name: &str, path_name: &str) -> Ext2Result {
        if self.file_type() != EXT2_FT_DIR {
            return Err(Ext2Error::NotADir);
        }
        debug!("symlink {} to {}", name, path_name);
        let inode = self.create(name, EXT2_S_IFLNK)?;
        if path_name.len() >= MAX_PATH_NAME {
            return Err(Ext2Error::PathTooLong);
        }
        inode.lock().append(path_name.as_bytes());
        Ok(())
    }

    fn ls_disk(&self, disk_inode: &DiskInode) -> Ext2Result<Vec<(String, DirEntryHead)>> {
        if !disk_inode.is_dir() {
            return Err(Ext2Error::NotADir);
        }
        let mut buffer = [0_u8; MAX_NAME_LEN];
        let mut dir_entries: Vec<(String, DirEntryHead)> = Vec::new();

        let mut dir_entry_head = DirEntryHead::empty();
        let mut offset: usize = 0;

        while offset + size_of::<DirEntryHead>() < disk_inode.i_size as usize {
            assert_eq!(
                disk_inode.read_at(
                    offset,
                    dir_entry_head.as_bytes_mut(),
                    &self.get_fs().manager,
                    Some(&self.blocks)
                ),
                size_of::<DirEntryHead>()
            );
            let name_len = dir_entry_head.name_len as usize;
            let name_buffer = &mut buffer[0..name_len];
            let name_offset = offset + size_of::<DirEntryHead>();
            assert_eq!(
                disk_inode.read_at(
                    name_offset,
                    name_buffer,
                    &self.get_fs().manager,
                    Some(&self.blocks)
                ),
                name_len
            );
            dir_entries.push((
                String::from_utf8_lossy(name_buffer).to_string(),
                dir_entry_head,
            ));
            offset += dir_entry_head.rec_len as usize;
        }

        Ok(dir_entries)
    }

    pub(crate) fn ls(&self) -> Ext2Result<Vec<String>> {
        if self.file_type() != EXT2_FT_DIR {
            return Err(Ext2Error::NotADir);
        }
        let dir_entries = self.read_disk_inode(|disk_inode| self.ls_disk(disk_inode));
        dir_entries.map(|inner| inner.into_iter().map(|(name, _)| name).collect())
    }

    pub(crate) fn ls_direntry(&self) -> Ext2Result<Vec<(String, DirEntryHead)>> {
        if self.file_type() != EXT2_FT_DIR {
            return Err(Ext2Error::NotADir);
        }
        self.read_disk_inode(|disk_inode| self.ls_disk(disk_inode))
    }

    fn is_empty_dir_disk(&self, disk_inode: &DiskInode) -> Ext2Result<bool> {
        if !disk_inode.is_dir() {
            return Err(Ext2Error::NotADir);
        }

        let mut dir_entry_head = DirEntryHead::empty();
        let mut offset: usize = 0;
        let mut file_num = 0;

        while offset + size_of::<DirEntryHead>() < disk_inode.i_size as usize {
            assert_eq!(
                disk_inode.read_at(
                    offset,
                    dir_entry_head.as_bytes_mut(),
                    &self.get_fs().manager,
                    Some(&self.blocks)
                ),
                size_of::<DirEntryHead>()
            );
            offset += dir_entry_head.rec_len as usize;
            file_num += 1;
            if file_num > 2 {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub(crate) fn is_empty_dir(&self) -> Ext2Result<bool> {
        if self.file_type() != EXT2_FT_DIR {
            return Err(Ext2Error::NotADir);
        }
        self.read_disk_inode(|disk_inode| self.is_empty_dir_disk(disk_inode))
    }

    fn unlink_below(&mut self) {
        assert!(self.file_type() == EXT2_FT_DIR);
        let names = self.ls().unwrap();

        for file_name in names.iter() {
            if file_name.as_str() == "." || file_name.as_str() == ".." {
                // special case
                continue;
            }
            let child_inode = self.find(file_name.as_str()).unwrap();
            let mut lk = child_inode.lock();
            if lk.file_type() == EXT2_FT_DIR {
                lk.unlink_below();
                lk.decrease_nlink(1);
                self.decrease_nlink(1);
            }
            drop(lk);
            self.unlink_single(file_name.as_str());
        }

        // when reaching here, it is assumed to be an empty directory
        self.read_disk_inode(|disk_inode| {
            assert!(disk_inode.i_links_count == 2);
        });
    }

    /// unlink recursively
    pub(crate) fn unlink(&mut self, name: &str, expect: u8, recursive: bool) -> Ext2Result {
        if self.file_type() != EXT2_FT_DIR {
            return Err(Ext2Error::NotADir);
        }
        debug!("unlink {}", name);
        if name == "." || name == ".." {
            error!("Can not unlink . or ..");
            return Err(Ext2Error::NotFound);
        }
        let inode = self.find(name)?;
        let mut lk = inode.lock();
        if lk.file_type() != expect {
            if expect == EXT2_FT_DIR {
                return Err(Ext2Error::NotADir);
            } else {
                return Err(Ext2Error::NotAFile);
            }
        }
        if lk.file_type() == EXT2_FT_DIR {
            let file_under_dir = lk.ls().unwrap();
            debug!("under this dir: {:?}", file_under_dir);
            if !lk.is_empty_dir().unwrap() && !recursive {
                return Err(Ext2Error::DirectoryIsNotEmpty);
            }
            lk.unlink_below();
            lk.decrease_nlink(1);
            self.decrease_nlink(1);
        }
        drop(lk);
        self.unlink_single(name);
        Ok(())
    }

    fn unlink_single(&mut self, name: &str) -> bool {
        assert!(self.file_type() == EXT2_FT_DIR);
        if name == "." || name == ".." {
            return false;
        }
        if let Some((de, pos, prev_offset)) = self.get_inode_id(name) {
            assert!(pos != 0);
            let mut buf = [0_u8; size_of::<DirEntryHead>()];
            self.read_at(prev_offset, &mut buf);
            unsafe {
                (*(&mut buf as *mut u8 as *mut DirEntryHead)).rec_len += de.rec_len;
            }
            self.write_at(prev_offset, &buf);

            let target_inode =
                Ext2FileSystem::get_inode_cache(&self.get_fs(), de.inode as usize).unwrap();
            target_inode.lock().decrease_nlink(1);
            true
        } else {
            false
        }
    }

    // ----- ACL ------
    pub(crate) fn chown(&self, uid: Option<usize>, gid: Option<usize>) {
        self.modify_disk_inode(|disk_inode| {
            if let Some(uid) = uid {
                disk_inode.i_uid = uid as _;
            }
            if let Some(gid) = gid {
                disk_inode.i_gid = gid as _;
            }
            disk_inode.i_mtime = self.get_fs().timer.get_current_time();
        })
    }
    pub(crate) fn chmod(&self, access: IMODE) {
        self.modify_disk_inode(|disk_inode| {
            disk_inode.i_mode = (disk_inode.i_mode & 0o7000) | access.bits();
            let cur_time = self.get_fs().timer.get_current_time();
            disk_inode.i_ctime = cur_time;
            disk_inode.i_atime = cur_time;
        });
    }

    // ----- Basic operation -----
    pub(crate) fn ftruncate(&mut self, new_size: u32) -> Ext2Result {
        if self.file_type() != EXT2_FT_REG_FILE {
            return Err(Ext2Error::NotAFile);
        }
        debug!("ftruncate from {} to {}", self.size, new_size);
        // if self.size < new_size as _ {
        //     self.cache_increase_size(new_size);
        // } else if self.size > new_size as _ {
        //     self.cache_decrease_size(new_size);
        // }
        match self.size.cmp(&(new_size as _)) {
            Ordering::Less => self.cache_increase_size(new_size),
            Ordering::Greater => self.cache_decrease_size(new_size),
            _ => (),
        }
        Ok(())
    }

    fn decrease_nlink(&mut self, by: usize) {
        let mut clean = false;
        self.modify_disk_inode(|disk_inode| {
            assert!(disk_inode.i_links_count >= by as u16);
            disk_inode.i_links_count -= by as u16;
            clean = disk_inode.i_links_count == 0;
        });

        if clean {
            self.clear();
            self.get_fs().dealloc_inode(self.inode_id as u32);
            self.get_fs()
                .inode_manager
                .lock()
                .try_to_remove(self.inode_id);
            self.valid = false;
        }
    }

    pub(crate) fn increase_nlink(&self, by: usize) {
        self.modify_disk_inode(|disk_inode| {
            disk_inode.i_links_count += by as u16;
        });
    }

    fn cache_increase_size(&mut self, new_size: u32) {
        if new_size <= self.size as _ {
            return;
        }
        let extra_blocks =
            self.modify_disk_inode(|disk_inode| self.increase_size(new_size, disk_inode));
        for block in extra_blocks {
            self.blocks.push(block);
        }
        self.size = new_size as _;
    }

    fn cache_decrease_size(&mut self, new_size: u32) {
        if new_size >= self.size as _ {
            return;
        }
        let remain_blocks =
            self.modify_disk_inode(|disk_inode| self.decrease_size(new_size, disk_inode));
        self.blocks.drain(remain_blocks..self.blocks.len());
        self.size = new_size as _;
    }

    /// Increase the size of a disk inode
    fn increase_size(&self, new_size: u32, disk_inode: &mut DiskInode) -> Vec<u32> {
        let blocks_needed = disk_inode.blocks_num_needed(new_size);
        let new_blocks = self.get_fs().batch_alloc_data(blocks_needed as _);
        assert!(new_blocks.len() == blocks_needed as _);
        disk_inode.increase_size(new_size, new_blocks, &self.get_fs().manager)
    }

    /// Decrease the size of a disk node
    fn decrease_size(&self, new_size: u32, disk_inode: &mut DiskInode) -> usize {
        let blocks_unused = disk_inode.decrease_size(new_size, &self.get_fs().manager);
        self.get_fs().batch_dealloc_block(&blocks_unused);
        disk_inode.data_blocks() as usize
    }
    /// Clear the data in current inode
    /// # Safety
    ///
    /// The inodecache should be marked as invalid and removed from cache manager right away
    pub(crate) fn clear(&self) {
        self.modify_disk_inode(|disk_inode| {
            let blocks = disk_inode.i_blocks;
            let data_blocks_dealloc = disk_inode.clear_size(&self.get_fs().manager);
            if data_blocks_dealloc.len() != DiskInode::total_blocks(blocks * 512) as usize {
                error!(
                    "clear: {} != {}",
                    data_blocks_dealloc.len(),
                    DiskInode::total_blocks(blocks * 512) as usize
                );
            }
            assert!(data_blocks_dealloc.len() == DiskInode::total_blocks(blocks * 512) as usize);
            let cur_time = self.get_fs().timer.get_current_time();
            disk_inode.i_atime = cur_time;
            disk_inode.i_mtime = cur_time;
            self.get_fs().batch_dealloc_block(&data_blocks_dealloc);
        });
    }
    /// Read data from current inode
    pub(crate) fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        self.modify_disk_inode(|disk_inode| {
            let cur_time = self.get_fs().timer.get_current_time();
            disk_inode.i_atime = cur_time;
            disk_inode.read_at(offset, buf, &self.get_fs().manager, Some(&self.blocks))
        })
    }
    /// Write data to current inode
    pub(crate) fn write_at(&mut self, offset: usize, buf: &[u8]) -> usize {
        self.cache_increase_size((offset + buf.len()) as _);
        self.modify_disk_inode(|disk_inode| {
            let cur_time = self.get_fs().timer.get_current_time();
            disk_inode.i_atime = cur_time;
            disk_inode.i_mtime = cur_time;
            disk_inode.write_at(offset, buf, &self.get_fs().manager, Some(&self.blocks))
        })
    }
    /// Write data at the end of file
    pub(crate) fn append(&mut self, buf: &[u8]) -> usize {
        let origin_size = self.size;
        self.cache_increase_size((origin_size + buf.len()) as _);
        self.modify_disk_inode(|disk_inode| {
            // let origin_size = disk_inode.i_size as usize;
            // self.increase_size((origin_size + buf.len()) as u32, disk_inode);
            let cur_time = self.get_fs().timer.get_current_time();
            disk_inode.i_atime = cur_time;
            disk_inode.i_mtime = cur_time;
            disk_inode.write_at(origin_size, buf, &self.get_fs().manager, Some(&self.blocks))
        })
    }
    pub(crate) fn append_dir_entry(&mut self, inode: usize, name: &str, file_type: u8) {
        let dir_entry = DirEntryHead::create(inode, name, file_type);
        self.append(dir_entry.as_bytes());
        let name_len = name.as_bytes().len();
        self.append(&name.as_bytes()[0..name_len.min(MAX_NAME_LEN)]);
    }
}

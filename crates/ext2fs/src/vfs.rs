use core::mem::size_of;
use log::*;

use crate::mutex::SpinMutex;

use super::{
    DiskInode, 
    Ext2FileSystem, layout::{
        MAX_NAME_LEN, DirEntryHead, EXT2_FT_UNKNOWN, EXT2_FT_DIR, EXT2_FT_REG_FILE,
        DEFAULT_IMODE, EXT2_S_IFDIR, EXT2_S_IFLNK, IMODE
    }
};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;

pub struct Inode {
    file_type: u8,
    inner: Arc<SpinMutex<InodeCache>>
}

impl Inode {
    pub fn new(inner: Arc<SpinMutex<InodeCache>>) -> Inode {
        let file_type = inner.lock().file_type();
        Self {
            file_type,
            inner
        }
    }

    fn access(&self) -> Option<&Arc<SpinMutex<InodeCache>>> {
        if self.inner.lock().valid {
            Some(&self.inner)
        } else {
            None
        }
    }

    // common operations

    pub fn file_type(&self) -> u8 {
        self.file_type
    }

    pub fn inode_id(&self) -> Option<usize> {
        Some(self.access()?.lock().inode_id)
    }

    pub fn chown(&self, uid: Option<usize>, gid:Option<usize>) -> Option<()> {
        Some(self.access()?.lock().chown(uid, gid))
    }

    pub fn chmod(&self, access: IMODE) -> Option<()> {
        Some(self.access()?.lock().chmod(access))
    }

    pub fn disk_inode(&self) -> Option<DiskInode> {
        Some(self.access()?.lock().disk_inode())
    }

    // file operation

    pub fn ftruncate(&self, new_size: usize) -> Option<bool> {
        Some(self.access()?.lock().ftruncate(new_size as _))
        
    }

    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> Option<usize> {
        let lk = self.access()?.lock();
        if self.file_type != EXT2_FT_REG_FILE {
            None
        } else {
            Some(lk.read_at(offset, buf))
        }
    }

    pub fn write_at(&self, offset: usize, buf: &[u8]) -> Option<usize> {
        let mut lk = self.access()?.lock();
        if self.file_type != EXT2_FT_REG_FILE {
            None
        } else {
            Some(lk.write_at(offset, buf))
        }
    }

    pub fn append(&self, buf: &[u8]) ->Option<usize> {
        let mut lk = self.access()?.lock();
        if self.file_type != EXT2_FT_REG_FILE {
            None
        } else {
            Some(lk.append(buf))
        }
    }

    // dir operation

    pub fn find(&self, name: &str) -> Option<Self> {
        let lk = self.access()?.lock();
        lk.find(name)
            .map(|inner| Self::new(inner))
    }

    pub fn create(&self, name: &str, file_type: u16) -> Option<Self> {
        let mut lk = self.access()?.lock();
        lk.create(name, file_type)
            .map(|inner| Self::new(inner))
    }

    pub fn ls(&self) -> Option<Vec<String>> {
        let lk = self.access()?.lock();
        if self.file_type != EXT2_FT_DIR {
            None
        } else {
            Some(lk.ls())
        }
    }

    pub fn is_empty_dir(&self) -> Option<bool> {
        let lk = self.access()?.lock();
        if self.file_type != EXT2_FT_DIR {
            None
        } else {
            Some(lk.is_empty_dir())
        }
    }

    pub fn link(&self, name: &str, inode_id: usize) -> Option<bool> {
        let mut lk = self.access()?.lock();
        Some(lk.link(name, inode_id))
    }

    pub fn symlink(&self, name: &str, path_name: &str) -> Option<bool> {
        let mut lk = self.access()?.lock();
        if self.file_type != EXT2_FT_DIR {
            None
        } else {
            Some(lk.symlink(name, path_name))
        }
    }

    pub fn rm_file(&self, file_name: &str) -> Option<bool> {
        let mut lk = self.access()?.lock();
        if self.file_type != EXT2_FT_DIR {
            None
        } else {
            Some(lk.unlink(file_name, EXT2_FT_REG_FILE, false))
        }
    }

    pub fn rm_dir(&self, dir_name: &str, recursive: bool) -> Option<bool> {
        let mut lk = self.access()?.lock();
        if self.file_type != EXT2_FT_DIR {
            None
        } else {
            Some(lk.unlink(dir_name, EXT2_FT_DIR, recursive))
        }
    }

}

/// Virtual filesystem layer over easy-fs
pub struct InodeCache {
    pub inode_id: usize,
    block_id: usize,
    block_offset: usize,
    fs: Arc<Ext2FileSystem>,

    // cache part
    file_type: u8,
    size: usize,
    blocks: Vec<u32>,
    pub valid: bool,
}

impl InodeCache {
    pub fn new(
        inode_id: usize,
        block_id: usize,
        block_offset: usize,
        fs: Arc<Ext2FileSystem>
    ) -> Self {
        let mut inode = Self {
            inode_id,
            block_id,
            block_offset,
            fs,
            file_type: EXT2_FT_UNKNOWN,
            size: 0,
            blocks: Vec::new(),
            valid: true
        };
        inode.read_cache();
        inode
    }

    /// Call a function over a disk inode to read it
    fn read_disk_inode<V>(&self, f: impl FnOnce(&DiskInode) -> V) -> V {
        let inode_block = self.fs.manager.lock().get_block_cache(self.block_id);
        let ret = inode_block.lock()
            .read(self.block_offset, f);
        self.fs.manager.lock().release_block(inode_block);
        ret
    }
    /// Call a function over a disk inode to modify it
    fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut DiskInode) -> V) -> V {
        let inode_block = self.fs.manager.lock().get_block_cache(self.block_id);
        let ret = inode_block.lock()
            .modify(self.block_offset, f);
        self.fs.manager.lock().release_block(inode_block);
        ret
    }

    pub fn read_cache(&mut self) {
        let mut file_type: u8 = 0;
        let mut file_size: usize = 0;
        let mut blocks: Vec<u32> = Vec::new();

        self.read_disk_inode(|disk_inode| {
            file_type = disk_inode.file_code();
            file_size = disk_inode.i_size as usize;
            blocks = disk_inode.all_data_blocks(&self.fs.manager, false);
        });

        self.file_type = file_type;
        self.size = file_size;
        self.blocks = blocks;
    }

    pub fn file_type(&self) -> u8 {
        self.file_type
    }

    pub fn disk_inode(&self) -> DiskInode {
        self.read_disk_inode(|disk_inode| *disk_inode)
    }

    /// Find inode under a disk inode by name (DirEntry, pos, prev_offset)
    fn find_inode_id(&self, name: &str, disk_inode: &DiskInode) -> Option<(DirEntryHead, usize, usize)> {
        // debug!("find_inode_id");
        // assert it is a directory
        assert!(disk_inode.is_dir());
        let mut buffer = [0 as u8; MAX_NAME_LEN];
        let mut dir_entry_head = DirEntryHead::empty();
        let mut offset: usize = 0;
        let mut pos: usize = 0;
        let mut prev_offset: usize = 0;

        while offset + size_of::<DirEntryHead>() < disk_inode.i_size as usize {
            assert_eq!(disk_inode.read_at(offset, dir_entry_head.as_bytes_mut(), &self.fs.manager, Some(&self.blocks)),
                        size_of::<DirEntryHead>());
            let name_len = dir_entry_head.name_len as usize;
            let name_buffer = &mut buffer[0..name_len];
            let name_offset = offset + size_of::<DirEntryHead>();
            assert_eq!(disk_inode.read_at(name_offset, name_buffer, &self.fs.manager, Some(&self.blocks)),
                        name_len);
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
        self.read_disk_inode(|disk_inode| {
            self.find_inode_id(name, disk_inode)
        })
    }

    pub fn find(&self, name: &str) -> Option<Arc<SpinMutex<InodeCache>>> {
        if let Some(de) = self.get_inode_id(name)
                                .map(|(de, _, _)| de)
        {
            Some(Ext2FileSystem::get_inode_cache(&self.fs, de.inode as _).unwrap())
        } else {
            None
        }
    }

    pub fn create(&mut self, name: &str, mut file_type: u16) -> Option<Arc<SpinMutex<InodeCache>>> {
        assert!(self.file_type() == EXT2_FT_DIR);
        if self.get_inode_id(name).is_some() {
            error!("Try to create a file already exists");
            return None;
        }
        file_type &= 0xF000;
        let new_inode_id = self.fs.alloc_inode().unwrap();
        let (new_inode_block_id, new_inode_block_offset) = self.fs.get_disk_inode_pos(new_inode_id);
        let inode_block = self.fs.manager.lock().get_block_cache(new_inode_block_id as _);
        inode_block.lock()
            .modify(new_inode_block_offset, |disk_inode: &mut DiskInode| {
                *disk_inode = DiskInode::new(DEFAULT_IMODE, file_type, 0, 0);
                let cur_time = self.fs.timer.get_current_time();
                disk_inode.i_atime = cur_time;
                disk_inode.i_ctime = cur_time;
            });
        self.fs.manager.lock().release_block(inode_block);

        let new_inode = Ext2FileSystem::get_inode_cache(&self.fs, new_inode_id as usize).unwrap();
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

        self.fs.write_meta();
        Some(new_inode)
    }

    /// can only link to file
    pub fn link(&mut self, name: &str, inode_id: usize) -> bool {
        assert!(self.file_type() == EXT2_FT_DIR);
        debug!("link {} to {}", name, inode_id);
        if inode_id == 0 {
            return false;
        }

        // link to self
        if self.inode_id == inode_id {
            return false;
        }

        if let Some(inode) = Ext2FileSystem::get_inode_cache(&self.fs, inode_id) {
            if self.get_inode_id(name).is_some() {
                // already exists
                false
            } else {
                let lk = inode.lock();
                if lk.file_type() != EXT2_FT_REG_FILE {
                    return false;
                }
                self.append_dir_entry(inode_id, name, EXT2_FT_REG_FILE);
                lk.increase_nlink(1);
                self.fs.write_meta();
                true
            }
        } else {
            false
        }
    }

    pub fn symlink(&mut self, name: &str, path_name: &str) -> bool {
        assert!(self.file_type() == EXT2_FT_DIR);
        debug!("symlink {} to {}", name, path_name);
        if let Some(inode) = self.create(name, EXT2_S_IFLNK) {
            inode.lock().append(path_name.as_bytes());
            true
        } else {
            false
        }
    }

    fn ls_disk(&self, disk_inode: &DiskInode) -> Vec<String> {
        assert!(disk_inode.is_dir());
        let mut buffer = [0 as u8; MAX_NAME_LEN];
        let mut names: Vec<String> = Vec::new();

        let mut dir_entry_head = DirEntryHead::empty();
        let mut offset: usize = 0;

        while offset + size_of::<DirEntryHead>() < disk_inode.i_size as usize {
            assert_eq!(disk_inode.read_at(offset, dir_entry_head.as_bytes_mut(), &self.fs.manager, Some(&self.blocks)),
                        size_of::<DirEntryHead>());
            let name_len = dir_entry_head.name_len as usize;
            let name_buffer = &mut buffer[0..name_len];
            let name_offset = offset + size_of::<DirEntryHead>();
            assert_eq!(disk_inode.read_at(name_offset, name_buffer, &self.fs.manager, Some(&self.blocks)),
                        name_len);
            names.push(String::from_utf8_lossy(name_buffer).to_string());
            offset += dir_entry_head.rec_len as usize;
        };

        names
    }

    pub fn ls(&self) -> Vec<String> {
        assert!(self.file_type() == EXT2_FT_DIR);
        self.read_disk_inode(|disk_inode| {
            self.ls_disk(disk_inode)
        })
    }

    fn is_empty_dir_disk(&self, disk_inode: &DiskInode) -> bool {
        assert!(disk_inode.is_dir());

        let mut dir_entry_head = DirEntryHead::empty();
        let mut offset: usize = 0;
        let mut file_num = 0;

        while offset + size_of::<DirEntryHead>() < disk_inode.i_size as usize {
            assert_eq!(disk_inode.read_at(offset, dir_entry_head.as_bytes_mut(), &self.fs.manager, Some(&self.blocks)),
                        size_of::<DirEntryHead>());
            offset += dir_entry_head.rec_len as usize;
            file_num += 1;
            if file_num > 2 {
                return false;
            }
        };

        true
    }

    pub fn is_empty_dir(&self) -> bool {
        assert!(self.file_type() == EXT2_FT_DIR);
        self.read_disk_inode(|disk_inode| {
            self.is_empty_dir_disk(disk_inode)
        })
    }

    fn unlink_below(&mut self) {
        assert!(self.file_type() == EXT2_FT_DIR);
        let names = self.ls();

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
    pub fn unlink(&mut self, name: &str, expect: u8, recursive: bool) -> bool {
        assert!(self.file_type() == EXT2_FT_DIR);
        debug!("unlink {}", name);
        if name == "." || name == ".." {
            error!("Can not unlink . or ..");
            return false;
        }

        if let Some(inode) = self.find(name) {
            let mut lk = inode.lock();
            if lk.file_type() != expect {
                return false;
            }
            if lk.file_type() == EXT2_FT_DIR {
                let file_under_dir = lk.ls();
                debug!("under this dir: {:?}", file_under_dir);
                if !lk.is_empty_dir() && !recursive {
                    return false;
                }
                lk.unlink_below();
                lk.decrease_nlink(1);
                self.decrease_nlink(1);
            }
            drop(lk);
            self.unlink_single(name);
            true
        } else {
            false
        }
    }

    fn unlink_single(&mut self, name: &str) -> bool {
        assert!(self.file_type() == EXT2_FT_DIR);
        if name == "." || name == ".." {
            return false;
        }
        if let Some((de, pos, prev_offset)) = self.get_inode_id(name) {
            assert!(pos != 0);
            let mut buf = [0 as u8; size_of::<DirEntryHead>()];
            self.read_at(prev_offset, &mut buf);
            unsafe {
                (*(&mut buf as *mut u8 as *mut DirEntryHead)).rec_len += de.rec_len;
            }
            self.write_at(prev_offset, &buf);
            
            let target_inode = Ext2FileSystem::get_inode_cache(&self.fs, de.inode as usize).unwrap();
            target_inode.lock().decrease_nlink(1);
            true
        } else {
            false
        }
    }

    // ----- ACL ------
    pub fn chown(&self, uid: Option<usize>, gid: Option<usize>) {
        self.modify_disk_inode(|disk_inode| {
            if let Some(uid) = uid {
                disk_inode.i_uid = uid as _;
            }
            if let Some(gid) = gid {
                disk_inode.i_gid = gid as _;
            }
            disk_inode.i_mtime = self.fs.timer.get_current_time();
        })
    }
    pub fn chmod(&self, access: IMODE) {
        self.modify_disk_inode(|disk_inode| {
            disk_inode.i_mode = (disk_inode.i_mode & 0o7000) | access.bits();
            let cur_time = self.fs.timer.get_current_time();
            disk_inode.i_ctime = cur_time;
            disk_inode.i_atime = cur_time;
        });
    }

    // ----- Basic operation -----
    pub fn ftruncate(&mut self, new_size: u32) -> bool {
        assert!(self.file_type() == EXT2_FT_REG_FILE);
        debug!("ftruncate from {} to {}", self.size, new_size);
        if self.size < new_size as _ {
            self.cache_increase_size(new_size);
        } else if self.size > new_size as _ {
            self.cache_decrease_size(new_size);
        }
        true
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
            self.fs.dealloc_inode(self.inode_id as u32);
            self.fs.inode_manager.lock().try_to_remove(self.inode_id);
            self.valid = false;
        }
    }

    pub fn increase_nlink(&self, by: usize) {
        self.modify_disk_inode(|disk_inode| {
            disk_inode.i_links_count += by as u16;
        });
    }

    fn cache_increase_size(&mut self, new_size: u32) {
        if new_size <= self.size as _{
            return;
        }
        let extra_blocks = self.modify_disk_inode(|disk_inode| {
            self.increase_size(new_size, disk_inode)
        });
        for block in extra_blocks {
            self.blocks.push(block);
        }
        self.size = new_size as _;
    }

    fn cache_decrease_size(&mut self, new_size: u32) {
        if new_size >= self.size as _ {
            return;
        }
        let remain_blocks = self.modify_disk_inode(|disk_inode| {
            self.decrease_size(new_size, disk_inode)
        });
        self.blocks.drain(remain_blocks..self.blocks.len());
        self.size = new_size as _;
    }

    /// Increase the size of a disk inode
    fn increase_size(
        &self,
        new_size: u32,
        disk_inode: &mut DiskInode,
    ) -> Vec<u32> {
        let blocks_needed = disk_inode.blocks_num_needed(new_size);
        let new_blocks = self.fs.batch_alloc_data(blocks_needed as _);
        assert!(new_blocks.len() == blocks_needed as _);
        disk_inode.increase_size(new_size, new_blocks, &self.fs.manager)
    }

    /// Decrease the size of a disk node
    fn decrease_size(
        &self,
        new_size: u32,
        disk_inode: &mut DiskInode,
    ) -> usize {
        let blocks_unused = disk_inode.decrease_size(new_size, &self.fs.manager);
        self.fs.batch_dealloc_block(&blocks_unused);
        return disk_inode.data_blocks() as usize;
    }
    /// Clear the data in current inode
    /// # Safety
    /// 
    /// The inodecache should be marked as invalid and removed from cache manager right away
    pub fn clear(&self) {
        self.modify_disk_inode(|disk_inode| {
            let blocks = disk_inode.i_blocks;
            let data_blocks_dealloc = disk_inode.clear_size(&self.fs.manager);
            if data_blocks_dealloc.len() != DiskInode::total_blocks(blocks * 512) as usize {
                error!("clear: {} != {}", data_blocks_dealloc.len(), DiskInode::total_blocks(blocks * 512) as usize);
            }
            assert!(data_blocks_dealloc.len() == DiskInode::total_blocks(blocks * 512) as usize);
            let cur_time = self.fs.timer.get_current_time();
            disk_inode.i_atime = cur_time;
            disk_inode.i_mtime = cur_time;
            self.fs.batch_dealloc_block(&data_blocks_dealloc);
        });
    }
    /// Read data from current inode
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        self.modify_disk_inode(|disk_inode| {
            let cur_time = self.fs.timer.get_current_time();
            disk_inode.i_atime = cur_time;
            disk_inode.read_at(offset, buf, &self.fs.manager, Some(&self.blocks))
        })
    }
    /// Write data to current inode
    pub fn write_at(&mut self, offset: usize, buf: &[u8]) -> usize {
        self.cache_increase_size((offset + buf.len()) as _);
        let size = self.modify_disk_inode(|disk_inode| {
            let cur_time = self.fs.timer.get_current_time();
            disk_inode.i_atime = cur_time;
            disk_inode.i_mtime = cur_time;
            disk_inode.write_at(offset, buf, &self.fs.manager, Some(&self.blocks))
        });
        size
    }
    /// Write data at the end of file
    pub fn append(&mut self, buf: &[u8]) -> usize {
        let origin_size = self.size;
        self.cache_increase_size((origin_size + buf.len()) as _);
        let size = self.modify_disk_inode(|disk_inode| {
            // let origin_size = disk_inode.i_size as usize;
            // self.increase_size((origin_size + buf.len()) as u32, disk_inode);
            let cur_time = self.fs.timer.get_current_time();
            disk_inode.i_atime = cur_time;
            disk_inode.i_mtime = cur_time;
            disk_inode.write_at(origin_size, buf, &self.fs.manager, Some(&self.blocks))
        });
        size
    }
    pub fn append_dir_entry(&mut self, inode: usize, name: &str, file_type: u8) {
        let dir_entry = DirEntryHead::create(inode, name, file_type);
        self.append(dir_entry.as_bytes());
        let name_len = name.as_bytes().len();
        self.append(&name.as_bytes()[0..name_len.min(MAX_NAME_LEN)]);
    }
}
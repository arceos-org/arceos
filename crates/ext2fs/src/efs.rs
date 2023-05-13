#![allow(unused)]
use crate::{block_cache_manager::BlockCacheManager, layout::EXT2_FT_DIR};
use crate::mutex::SpinMutex;
use crate::timer::TimeProvider;
use crate::inode_manager::InodeCacheManager;
use core::mem::size_of;
use fs_utils::sync::Spin;
use log::*;

use super::{
    Bitmap, BlockDevice, DiskInode, BlockGroupDesc, InodeCache, Inode,
    SuperBlock, config::{
        BLOCK_SIZE, BLOCKS_PER_GRP, RESERVED_BLOCKS_PER_GRP, EXT2_ROOT_INO,
        FIRST_DATA_BLOCK, INODES_PER_GRP, EXT2_GOOD_OLD_FIRST_INO, SUPER_BLOCK_OFFSET
    },
    layout::{IMODE, EXT2_S_IFDIR, EXT2_S_IFREG}
};
use alloc::{sync::Arc, vec::Vec};
use spin::Mutex;

pub struct Ext2FileSystem {
    ///Real device
    pub manager: SpinMutex<BlockCacheManager>,
    /// manage inode cache
    pub inode_manager: SpinMutex<InodeCacheManager>,
    /// provide time
    pub timer: Arc<dyn TimeProvider>,
    /// inner meta data
    inner: Mutex<Ext2FileSystemInner>
}

type DataBlock = [u8; BLOCK_SIZE];

const MAX_CACHE_NUM: usize = 50;

impl Ext2FileSystem {
    /// Create an ext2 file system in a device
    pub fn create(block_device: Arc<dyn BlockDevice>, timer: Arc<dyn TimeProvider>) -> Arc<Self> {
        assert!(block_device.block_size() == BLOCK_SIZE, "Unsupported block size");
        debug!("Create ext2 file system...");
        let mut block_num = block_device.block_num();
        let mut group_num = (block_num + BLOCKS_PER_GRP - 1)/BLOCKS_PER_GRP;
        assert!(group_num >= 1, "Size is at least 32 MB");
        let mut last_group_block_num = block_num - (group_num - 1) * BLOCKS_PER_GRP;

        if last_group_block_num <= RESERVED_BLOCKS_PER_GRP {
            group_num -= 1;
            last_group_block_num = BLOCKS_PER_GRP;
        }
        assert!(group_num >= 1);
        block_num = (group_num - 1) * BLOCKS_PER_GRP + last_group_block_num;
        let group_desc_block_num = (group_num * size_of::<BlockGroupDesc>() + BLOCK_SIZE - 1)/BLOCK_SIZE;

        let mut group_desc_table:Vec<BlockGroupDesc> = Vec::new();
        for group_id in 0..group_num {
            let mut block_bitmap: usize = 0;
            let mut free_blocks: usize = 0;
            if group_id == 0 {
                block_bitmap = (FIRST_DATA_BLOCK + 1) + group_desc_block_num;
                free_blocks = if group_id == group_num - 1 {
                    last_group_block_num - block_bitmap - RESERVED_BLOCKS_PER_GRP
                } else {
                    BLOCKS_PER_GRP - block_bitmap - RESERVED_BLOCKS_PER_GRP
                }
                // first group
            } else if group_id == group_num - 1 {
                // last group
                block_bitmap = BLOCKS_PER_GRP * group_id;
                free_blocks = last_group_block_num - RESERVED_BLOCKS_PER_GRP;
            } else {
                block_bitmap = BLOCKS_PER_GRP * group_id;
                free_blocks = BLOCKS_PER_GRP - RESERVED_BLOCKS_PER_GRP;
            }
            group_desc_table.push(BlockGroupDesc::new(
                block_bitmap,
                block_bitmap + 1,
                block_bitmap + 2,
                free_blocks, INODES_PER_GRP, 0
            ));
        }

        let super_block = SuperBlock::new(
            INODES_PER_GRP * group_num,
            block_num,
            INODES_PER_GRP * group_num - EXT2_GOOD_OLD_FIRST_INO + 1,
            block_num - group_num * RESERVED_BLOCKS_PER_GRP - (FIRST_DATA_BLOCK + 1 + group_desc_block_num),
            group_num,
            "Image by hsh"
        );

        let mut cache_manager = BlockCacheManager::new();

        let fs = Arc::new(Self {
            manager: SpinMutex::new(cache_manager),
            inode_manager: SpinMutex::new(InodeCacheManager::new(64)),
            timer,
            inner: Mutex::new(Ext2FileSystemInner::new(super_block, group_desc_table))
        });
        fs.manager.lock().init(block_device.clone(), MAX_CACHE_NUM);

        // clear all blocks except the first 1024 bytes
        for i in 0..block_num {
            let block = fs.manager.lock().get_block_cache(i as _);
            block.lock()
                .modify(0, |data_block: &mut DataBlock| {
                    for (idx, byte) in data_block.iter_mut().enumerate() {
                        if i != 0 || idx >= 1024 {
                            *byte = 0;
                        }
                    }
                });
            fs.manager.lock().release_block(block);
        }

        // TODO: mark reserved inodes and used data blocks
        let mut inner = fs.inner.lock();
        debug!("Super block:\n {:?}", &inner.super_block);
        for (idx, desc) in inner.group_desc_table.iter().enumerate() {
            debug!("Block group {:?}:\n{:?}", idx, desc);
        }
        inner.get_inode_bitmap(0)
            .range_alloc(&fs.manager, 1, EXT2_GOOD_OLD_FIRST_INO);
        for group_id in 0..group_num {
            // debug!("Range alloc block in group {} {} {}", 
            //     group_id,
            //     group_id * BLOCKS_PER_GRP,
            //     fs.group_desc_table[group_id].bg_block_bitmap as usize + RESERVED_BLOCKS_PER_GRP + 1
            // );
            inner.get_data_bitmap(group_id)
                .range_alloc(
                    &fs.manager, 
                    group_id * BLOCKS_PER_GRP, 
                    inner.group_desc_table[group_id].bg_block_bitmap as usize + RESERVED_BLOCKS_PER_GRP + 1
                );
            if group_id == group_num - 1 {
                if block_num < (group_id + 1) * BLOCKS_PER_GRP {
                    inner.get_data_bitmap(group_id)
                        .range_alloc(
                            &fs.manager, 
                            block_num, 
                            (group_id + 1) * BLOCKS_PER_GRP
                        );
                }
            }
        }

        // TODO: init '/' inode
        let (root_inode_block_id, root_inode_offset) = inner.get_disk_inode_pos(EXT2_ROOT_INO as u32);
        let inode_block = fs.manager.lock().get_block_cache(root_inode_block_id as _);
        inode_block.lock()
            .modify(root_inode_offset, |disk_inode: &mut DiskInode| {
                *disk_inode = DiskInode::new(
                    IMODE::from_bits_truncate(0o755), 
                    EXT2_S_IFDIR, 0, 0);
            });
        fs.manager.lock().release_block(inode_block);

        drop(inner);
        // TODO: write super blocks and group description table to disk

        // TODO: create dir entry '.' and '..' for '/'
        let root_inode = Self::root_inode_cache(&fs);
        // root_inode.lock().link(".", EXT2_ROOT_INO);
        // root_inode.lock().link("..", EXT2_ROOT_INO);
        let mut lk = root_inode.lock();
        lk.append_dir_entry(EXT2_ROOT_INO, ".", EXT2_FT_DIR);
        lk.append_dir_entry(EXT2_ROOT_INO, "..", EXT2_FT_DIR);
        lk.increase_nlink(2);

        fs.write_meta();
        // fs.inner.lock().super_block.check_valid();
        fs.manager.lock().sync_all_block();
        fs
    }

    /// Open a file system from disk
    pub fn open(block_device: Arc<dyn BlockDevice>, timer: Arc<dyn TimeProvider>) -> Arc<Self> {
        assert!(block_device.block_size() == BLOCK_SIZE, "Unsupported block size");
        debug!("Open ext2 file system...");
        let fs = Arc::new(Self {
            manager: SpinMutex::new(BlockCacheManager::new()),
            inode_manager: SpinMutex::new(InodeCacheManager::new(64)),
            timer,
            inner: Mutex::new(Ext2FileSystemInner::new(SuperBlock::empty(), Vec::new()))
        });
        fs.manager.lock().init(block_device.clone(), MAX_CACHE_NUM);
        // get_block_cache(FIRST_DATA_BLOCK, Arc::clone(&block_device))
        //     .lock()
        //     .read(SUPER_BLOCK_OFFSET, |sb: &SuperBlock| {
        //         super_block = *sb;
        //     });
        debug!("After manager init");
        let sb_block = fs.manager.lock().get_block_cache(FIRST_DATA_BLOCK);
        sb_block.lock()
            .read(SUPER_BLOCK_OFFSET, |sb: &SuperBlock| {
                fs.inner.lock().super_block = *sb;
            });
        fs.manager.lock().release_block(sb_block);
        debug!("Super block:\n {:?}", &fs.inner.lock().super_block);
        fs.inner.lock().super_block.check_valid();
        debug!("After superblock check valid");
        
        let s_block_group_nr = fs.inner.lock().super_block.s_block_group_nr;
        let s_first_data_block = fs.inner.lock().super_block.s_first_data_block;

        for group_id in 0..s_block_group_nr as usize {
            let block_id = s_first_data_block as usize + 1 + (group_id * size_of::<BlockGroupDesc>())/BLOCK_SIZE;
            let offset = (group_id * size_of::<BlockGroupDesc>())%BLOCK_SIZE;
            // get_block_cache(block_id, Arc::clone(&block_device))
            //     .lock()
            //     .read(offset, |desc: &BlockGroupDesc| {
            //         group_desc_table.push(*desc);
            //     });
            let gdt_block = fs.manager.lock().get_block_cache(block_id);
            gdt_block.lock()
                .read(offset, |desc: &BlockGroupDesc| {
                    fs.inner.lock().group_desc_table.push(*desc);
                });
            fs.manager.lock().release_block(gdt_block);
        }
        let cur_time = fs.timer.get_current_time();

        {
            let mut inner = fs.inner.lock();
            inner.super_block.s_mnt_count += 1;
            inner.super_block.s_mtime = cur_time;
        }

        for (idx, desc) in fs.inner.lock().group_desc_table.iter().enumerate() {
            debug!("Block group {:?}:\n{:?}", idx, desc);
        }

        fs.write_super_block();

        fs
    }

    pub fn root_inode(efs: &Arc<Self>) -> Inode {
        Inode::new(Self::root_inode_cache(efs))
    }

    /// Get root inode
    fn root_inode_cache(efs: &Arc<Self>) -> Arc<SpinMutex<InodeCache>> {
        Self::get_inode_cache(efs, EXT2_ROOT_INO).unwrap()
    }

    pub fn get_inode_cache(efs: &Arc<Self>, inode_id: usize) -> Option<Arc<SpinMutex<InodeCache>>> {
        efs.inode_manager.lock().get_or_insert(inode_id, efs)
    } 

    pub fn create_inode_cache(efs: &Arc<Self>, inode_id: usize) -> Option<InodeCache> {
        if inode_id == 0 || !efs.inode_exists(inode_id as _) {
            None
        } else {
            let (block_id, offset) = efs.inner.lock().get_disk_inode_pos(inode_id as u32);
            Some(InodeCache::new(
                inode_id,
                block_id as usize,
                offset,
                Arc::clone(efs)
            ))
        }
    }

    /// Get inode block_id and offset from inode_id
    pub fn get_disk_inode_pos(&self, mut inode_id: u32) -> (u32, usize) {
        self.inner.lock().get_disk_inode_pos(inode_id)
    }

    // /// Get inode bitmap for group x
    // pub fn get_inode_bitmap(&self, group_id: usize) -> Bitmap {
    //     self.inner.lock().get_inode_bitmap(group_id)
    // }

    // /// Get data bitmap for group x
    // pub fn get_data_bitmap(&self, group_id: usize) -> Bitmap {
    //     self.inner.lock().get_data_bitmap(group_id)
    // }

    /// Allocate inode (will modify meta data)
    pub fn alloc_inode(&self) -> Option<u32> {
        let mut inner = self.inner.lock();
        for group_id in 0..inner.group_desc_table.len() {
            if let Some(inode_id) = inner.get_inode_bitmap(group_id).alloc(&self.manager) {
                inner.group_desc_table[group_id].bg_free_inodes_count -= 1; // still need to mantain bg_used_dir_count
                inner.super_block.s_free_inodes_count -= 1;
                return  Some(inode_id as u32);
            }
        }
        None
    }

    /// Allocate data block (will modify meta data)
    pub fn alloc_data(&self) -> Option<u32> {
        let mut inner = self.inner.lock();
        for group_id in 0..inner.group_desc_table.len() {
            if let Some(block_id) = inner.get_data_bitmap(group_id).alloc(&self.manager) {
                inner.group_desc_table[group_id].bg_free_blocks_count -= 1; // still need to mantain bg_used_dir_count
                inner.super_block.s_free_blocks_count -= 1;
                return Some(block_id as u32);
            }
        }
        None
    }

    /// Batch allocate data
    pub fn batch_alloc_data(&self, block_num: usize) -> Vec<u32> {
        let mut inner = self.inner.lock();
        let mut allocated_blocks: Vec<u32> = Vec::new();
        for _ in 0..block_num {
            let block_id = {
                let mut result = None;
                for group_id in 0..inner.group_desc_table.len() {
                    if let Some(block_id) = inner.get_data_bitmap(group_id).alloc(&self.manager) {
                        inner.group_desc_table[group_id].bg_free_blocks_count -= 1; // still need to mantain bg_used_dir_count
                        inner.super_block.s_free_blocks_count -= 1;
                        result = Some(block_id as u32);
                        break;
                    }
                }
                result
            };
            if let Some(bid) = block_id {
                allocated_blocks.push(bid);
            } else {
                return allocated_blocks;
            }
        }

        allocated_blocks
    }

    /// Test whether an inode exists
    pub fn inode_exists(&self, inode_id: u32) -> bool {
        assert!(inode_id != 0);
        let mut inner = self.inner.lock();
        let group_id = (inode_id as usize - 1) / INODES_PER_GRP;
        inner.get_inode_bitmap(group_id).test(&self.manager, inode_id as usize)
    }

    /// Dealloc inode (will modify meta data)
    pub fn dealloc_inode(&self, inode_id: u32) {
        assert!(inode_id != 0);
        let mut inner = self.inner.lock();
        let group_id = (inode_id as usize - 1) / INODES_PER_GRP;
        inner.get_inode_bitmap(group_id).dealloc(&self.manager, inode_id as usize);

        inner.super_block.s_free_inodes_count += 1;
        inner.group_desc_table[group_id].bg_free_inodes_count += 1;
    }

    /// Dealloc inode (will modify meta data)
    pub fn dealloc_block(&self, block_id: u32) {
        let target_block = self.manager.lock().get_block_cache(block_id as _);
        target_block.lock()
            .modify(0, |data_block: &mut DataBlock| {
                data_block.iter_mut().for_each(|p| {
                    *p = 0;
                })
            });
        self.manager.lock().release_block(target_block);
        let mut inner = self.inner.lock();
        let group_id = block_id as usize / BLOCKS_PER_GRP;
        inner.get_data_bitmap(group_id).dealloc(&self.manager, block_id as usize);
        inner.super_block.s_free_blocks_count += 1;
        inner.group_desc_table[group_id].bg_free_blocks_count += 1;
    }

    pub fn batch_dealloc_block(&self, blocks: &Vec<u32>) {
        let mut inner = self.inner.lock();
        for block_id in blocks {
            let target_block = self.manager.lock().get_block_cache(*block_id as _);
            target_block.lock()
                .modify(0, |data_block: &mut DataBlock| {
                    data_block.iter_mut().for_each(|p| {
                        *p = 0;
                    })
                });
            self.manager.lock().release_block(target_block);
            let group_id = *block_id as usize / BLOCKS_PER_GRP;
            inner.get_data_bitmap(group_id).dealloc(&self.manager, *block_id as usize);
            inner.super_block.s_free_blocks_count += 1;
            inner.group_desc_table[group_id].bg_free_blocks_count += 1;
        }
    }

    /// Write super block to disk
    pub fn write_super_block(&self) {
        self.inner.lock().write_super_block(&self.manager);
    }

    // /// Write group description of group_id to disk
    // pub fn write_group_desc(&self, group_id: usize) {
    //     let block_id = self.super_block.s_first_data_block as usize + 1 + (group_id * size_of::<BlockGroupDesc>())/BLOCK_SIZE;
    //     let offset = (group_id * size_of::<BlockGroupDesc>())%BLOCK_SIZE;
    //     get_block_cache(block_id, Arc::clone(&self.block_device))
    //         .lock()
    //         .modify(offset, |desc: &mut BlockGroupDesc| {
    //             *desc = self.group_desc_table[group_id];
    //         });
    // }

    // /// Write all group description to disk
    // pub fn write_all_group_desc(&self) {
    //     for group_id in 0..self.group_desc_table.len() {
    //         self.write_group_desc(group_id);
    //     }
    // }

    /// Write all meta data to disk
    pub fn write_meta(&self) {
        self.inner.lock().write_meta(&self.manager);
    }
}

impl Drop for Ext2FileSystem {
    fn drop(&mut self) {
        self.write_meta();
        self.manager.lock().sync_all_block();
    }
}

struct Ext2FileSystemInner {
    /// Super block cache
    pub super_block: SuperBlock,
    /// Group description
    pub group_desc_table: Vec<BlockGroupDesc>,
}

impl Ext2FileSystemInner {
    pub fn new(sb: SuperBlock, gdt: Vec<BlockGroupDesc>) -> Self {
        Self { super_block: sb, group_desc_table: gdt }
    }

    /// Get inode block_id and offset from inode_id
    pub fn get_disk_inode_pos(&self, mut inode_id: u32) -> (u32, usize) {
        assert!(inode_id != 0); // invalid inode id
        inode_id -= 1;
        let group_id = inode_id/INODES_PER_GRP as u32;
        let group_offset = inode_id%INODES_PER_GRP as u32;
        let inode_size = size_of::<DiskInode>();
        let inode_per_block = BLOCK_SIZE/inode_size;
        let block_id = self.group_desc_table[group_id as usize].bg_inode_table + group_offset/inode_per_block as u32;

        (block_id, (group_offset as usize%inode_per_block) * inode_size)
    }

    /// Get inode bitmap for group x
    pub fn get_inode_bitmap(&self, group_id: usize) -> Bitmap {
        Bitmap::new(
            self.group_desc_table[group_id].bg_inode_bitmap as usize,
            group_id * INODES_PER_GRP + 1
        )
    }

    /// Get data bitmap for group x
    pub fn get_data_bitmap(&self, group_id: usize) -> Bitmap {
        Bitmap::new(
            self.group_desc_table[group_id].bg_block_bitmap as usize,
            group_id * BLOCKS_PER_GRP
        )
    }

    /// Write super block to disk
    pub fn write_super_block(&self, manager: &SpinMutex<BlockCacheManager>) {
        let offset = if self.super_block.s_first_data_block == 0 { 1024 } else { 0 };
        let sb_block = manager.lock().get_block_cache(self.super_block.s_first_data_block as _);
        sb_block.lock()
            .modify(offset, |super_block: &mut SuperBlock| {
                *super_block = self.super_block;
            });
        manager.lock().release_block(sb_block);
    }

    /// Write group description of group_id to disk
    pub fn write_group_desc(&self, group_id: usize, manager: &SpinMutex<BlockCacheManager>) {
        let block_id = self.super_block.s_first_data_block as usize + 1 + (group_id * size_of::<BlockGroupDesc>())/BLOCK_SIZE;
        let offset = (group_id * size_of::<BlockGroupDesc>())%BLOCK_SIZE;
        let gd_block = manager.lock().get_block_cache(block_id);
        gd_block.lock()
            .modify(offset, |desc: &mut BlockGroupDesc| {
                *desc = self.group_desc_table[group_id];
            });
        manager.lock().release_block(gd_block);
    }

    /// Write all group description to disk
    pub fn write_all_group_desc(&self, manager: &SpinMutex<BlockCacheManager>) {
        for group_id in 0..self.group_desc_table.len() {
            self.write_group_desc(group_id, manager);
        }
    }

    /// Write all meta data to disk
    pub fn write_meta(&self, manager: &SpinMutex<BlockCacheManager>) {
        self.write_super_block(manager);
        self.write_all_group_desc(manager);
    }
}
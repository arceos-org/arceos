#![allow(unused)]
use super::{config::*};
use crate::block_cache_manager::{BlockCacheManager};
use crate::mutex::SpinMutex;
use _core::mem::size_of;
use bitflags::*;
use alloc::{string::String, vec::Vec};
use core::fmt::{Debug, Formatter, Result};
use log::*;

const VOLUMN_NAME_SIZE: usize = 16;
const MOUNT_SIZE: usize = 64;
const HASH_SEED_SIZE: usize = 4;
const SB_RESERVED_SIZE: usize = 760;

pub const DIRECT_BLOCK_NUM: usize = 13;
pub const DOUBLE_BLOCK_NUM: usize = BLOCK_SIZE/4;
pub const DOUBLE_BLOCK_BOUND: usize = DIRECT_BLOCK_NUM + DOUBLE_BLOCK_NUM;
pub const TRIPLE_BLOCK_NUM: usize = (BLOCK_SIZE/4) * (BLOCK_SIZE/4);
// pub const TRIPLE_BLOCK_BOUND: usize = TRIPLE_BLOCK_NUM + DOUBLE_BLOCK_BOUND;
pub const SB_MAGIC: u16 = 0xEF53;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct SuperBlock {
    pub s_inodes_count: u32,
    pub s_blocks_count: u32,
    s_r_blocks_count: u32,
    pub s_free_blocks_count: u32,
    pub s_free_inodes_count: u32,
    pub s_first_data_block: u32,
    s_log_block_size: u32,
    s_log_frag_size: u32,
    s_blocks_per_group: u32,
    s_frags_per_group: u32,
    s_inodes_per_group: u32,
    pub s_mtime: u32,
    pub s_wtime: u32,
    pub s_mnt_count: u16,
    s_max_mnt_count: u16,
    s_magic: u16,
    s_state: u16,
    s_errors: u16,
    s_minor_rev_level: u16,
    s_lastcheck: u32,
    s_checkinterval: u32,
    s_creator_os: u32,
    s_rev_level: u32,
    s_def_resuid: u16,
    s_def_resgid: u16,
    // EXT2_DYNAMIC_REV Specific
    s_first_ino: u32,
    s_inode_size: u16,
    pub s_block_group_nr: u16,
    s_feature_compat: FeatureCompat,
    s_feature_incompat: FeatureIncompat,
    s_feature_ro_compat: FeatureRocompat,
    s_uuid: u128,
    s_volume_name: [u8; VOLUMN_NAME_SIZE],
    s_last_mounted: [u8; MOUNT_SIZE],
    s_algo_bitmap: u32,
    // Performance hints
    s_prealloc_blocks: u8,
    s_prealloc_dir_blocks: u8,
    p_padding: [u8; 2],
    // Journaling Support
    s_journal_uuid: u128,
    s_journal_inum: u32,
    s_journal_dev: u32,
    s_last_orphan: u32,
    // Directory Indexing Support
    s_hash_seed: [u32; HASH_SEED_SIZE],
    s_def_hash_version: u8,
    i_padding: [u8; 3],
    // Other options
    s_default_mount_option: u32,
    s_first_meta_bg: u32,
    reserved: [u8; SB_RESERVED_SIZE]
}

// s_state
const EXT2_VALID_FS: u16 = 1;
const EXT2_ERROR_FS: u16 = 2;

// s_errors
const EXT2_ERRORS_CONTINUE: u16 = 1;
const EXT2_ERRORS_RO: u16 = 2;
const EXT2_ERRORS_PANIC: u16 = 3;

// s_creator_os
const EXT2_OS_LINUX: u32 = 0;
const EXT2_OS_HURD: u32 = 1;
const EXT2_OS_MASIX: u32 = 2;
const EXT2_OS_FREEBSD: u32 = 3;
const EXT2_OS_LITES: u32 = 4;

// s_rev_level
const EXT2_GOOD_OLD_REV: u32 = 0;
const EXT2_DYNAMIC_REV: u32 = 1;

// s_def_resuid
const EXT2_DEF_RESUID: u16 = 0;

// s_def_resgid
const EXT2_DEF_RESGID: u16 = 0;

// s_feature_compat

bitflags! {
    pub struct FeatureCompat: u32 {
        const EXT2_FEATURE_COMPAT_DIR_PREALLOC = 1;
        const EXT2_FEATURE_COMPAT_IMAGIC_INODES = 1 << 1;
        const EXT3_FEATURE_COMPAT_HAS_JOURNAL = 1 << 2;
        const EXT2_FEATURE_COMPAT_EXT_ATTR = 1 << 3;
        const EXT2_FEATURE_COMPAT_RESIZE_INO = 1 << 4;
        const EXT2_FEATURE_COMPAT_DIR_INDEX = 1 << 5;
    }
}

bitflags! {
    pub struct FeatureIncompat: u32 {
        const EXT2_FEATURE_INCOMPAT_COMPRESSION = 1;
        const EXT2_FEATURE_INCOMPAT_FILETYPE = 1 << 1;
        const EXT3_FEATURE_INCOMPAT_RECOVER = 1 << 2;
        const EXT3_FEATURE_INCOMPAT_JOURNAL_DEV = 1 << 3;
        const EXT2_FEATURE_INCOMPAT_META_BG = 1 << 4;
    }
}

bitflags! {
    pub struct FeatureRocompat: u32 {
        const EXT2_FEATURE_RO_COMPAT_SPARSE_SUPER = 1;
        const EXT2_FEATURE_RO_COMPAT_LARGE_FILE = 1 << 1;
        const EXT2_FEATURE_RO_COMPAT_BTREE_DIR = 1 << 2;
    }
}

bitflags! {
    pub struct AlgoBitmap: u32 {
        const EXT2_LZV1_ALG = 1;
        const EXT2_LZRW3A_ALG = 1 << 1;
        const EXT2_GZIP_ALG = 1 << 2;
        const EXT2_BZIP2_ALG = 1 << 3;
        const EXT2_LZO_ALG = 1 << 4;
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BlockGroupDesc {
    pub bg_block_bitmap: u32,
    pub bg_inode_bitmap: u32,
    pub bg_inode_table: u32,
    pub bg_free_blocks_count: u16,
    pub bg_free_inodes_count: u16,
    pub bg_used_dirs_count: u16,
    bg_pad: u16,
    bg_reserved: [u8; 12]
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct DiskInode {
    pub i_mode: u16,
    pub i_uid: u16,
    pub i_size: u32,
    pub i_atime: u32,
    pub i_ctime: u32,
    pub i_mtime: u32,
    pub i_dtime: u32,
    pub i_gid: u16,
    pub i_links_count: u16,
    // the total number of 512-bytes blocks
    pub i_blocks: u32,
    i_flags: u32,
    i_osd1: u32,
    i_direct_block: [u32; DIRECT_BLOCK_NUM],
    i_double_block: u32,
    i_triple_block: u32,
    i_generation: u32,
    i_file_acl: u32,
    i_dir_acl: u32,
    i_faddr: u32,
    i_osd2: LinuxOSD
}

/// A indirect block
type IndirectBlock = [u32; BLOCK_SIZE / 4];
/// A data block
type DataBlock = [u8; BLOCK_SIZE];

// Defined Reserved Inodes
const EXT2_BAD_INO: u32 = 1;
// const EXT2_ROOT_INO: u32 = 2; (from config)
const EXT2_ACL_IDX_INO: u32 = 3;
const EXT2_ACL_DATA_INO: u32 = 4;
const EXT2_BOOT_LOADER_INO: u32 = 5;
const EXT2_UNDEL_DIR_INO: u32 = 6;

bitflags! {
    pub struct IMODE: u16 {
        // access control
        const EXT2_S_IXOTH = 1;
        const EXT2_S_IWOTH = 1 << 1;
        const EXT2_S_IROTH = 1 << 2;
        const EXT2_S_IXGRP = 1 << 3;
        const EXT2_S_IWGRP = 1 << 4;
        const EXT2_S_IRGRP = 1 << 5;
        const EXT2_S_IXUSR = 1 << 6;
        const EXT2_S_IWUSR = 1 << 7;
        const EXT2_S_IRUSR = 1 << 8;
        // process
        const EXT2_S_ISVTX = 1 << 9;
        const EXT2_S_ISGID = 1 << 10;
        const EXT2_S_ISUID = 1 << 11;
    }
}

pub const DEFAULT_IMODE: IMODE = IMODE::from_bits_truncate(0o755); // rwxrw-rw-

// IMODE -> file format
const EXT2_S_IFIFO: u16 = 0x1000;
const EXT2_S_IFCHR: u16 = 0x2000;
pub const EXT2_S_IFDIR: u16 = 0x4000;
pub const EXT2_S_IFBLK: u16 = 0x6000;
pub const EXT2_S_IFREG: u16 = 0x8000;
pub const EXT2_S_IFLNK: u16 = 0xA000;
const EXT2_S_IFSOCK: u16 = 0xC000;

// DirEntry -> file code
pub const EXT2_FT_UNKNOWN: u8 = 0;
pub const EXT2_FT_REG_FILE: u8 = 1;
pub const EXT2_FT_DIR: u8 = 2;
const EXT2_FT_CHRDEV: u8 = 3;
const EXT2_FT_BLKDEV: u8 = 4;
const EXT2_FT_FIFO: u8 = 5;
const EXT2_FT_SOCK: u8 = 6;
pub const EXT2_FT_SYMLINK: u8 = 7;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LinuxOSD {
    l_i_frag: u8,
    l_i_fsize: u8,
    reserved: [u8; 2],
    l_i_uid_high: u16,
    l_i_gid_high: u16,
    reserved_2: [u8; 4]
}

impl LinuxOSD {
    fn empty() -> LinuxOSD {
        LinuxOSD {
            l_i_frag: 0,
            l_i_fsize: 0,
            reserved: [0; 2],
            l_i_uid_high: 0,
            l_i_gid_high: 0,
            reserved_2: [0; 4]
        }
    }
}

pub const MAX_NAME_LEN: usize = 255;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct DirEntryHead {
    pub inode: u32,
    pub rec_len: u16,
    pub name_len: u8,
    pub file_type: u8,
    // name is variable length
}

impl SuperBlock {
    pub fn new(
        inodes_count: usize,
        blocks_count: usize,
        free_inodes_count: usize,
        free_blocks_count: usize,
        block_group_num: usize,
        volumn_name: &str
    ) -> SuperBlock 
    {
        let mut sb = SuperBlock {
            s_inodes_count: inodes_count as u32,
            s_blocks_count: blocks_count as u32,
            s_r_blocks_count: 0,
            s_free_blocks_count: free_blocks_count as u32,
            s_free_inodes_count: free_inodes_count as u32,
            s_first_data_block: FIRST_DATA_BLOCK as u32,
            s_log_block_size: LOG_BLOCK_SIZE as u32,
            s_log_frag_size: LOG_FRAG_SIZE as u32,
            s_blocks_per_group: BLOCKS_PER_GRP as u32,
            s_frags_per_group: BLOCKS_PER_GRP as u32,
            s_inodes_per_group: INODES_PER_GRP as u32,
            s_mtime: FAKE_CREATE_TIME as u32,
            s_wtime: FAKE_CREATE_TIME as u32,
            s_mnt_count: 0,
            s_max_mnt_count: 32,
            s_magic: SB_MAGIC,
            s_state: EXT2_VALID_FS,
            s_errors: EXT2_ERRORS_RO,
            s_minor_rev_level: 0,
            s_lastcheck: FAKE_CREATE_TIME as u32,
            s_checkinterval: CHECK_INTERVAL as u32,
            s_creator_os: EXT2_OS_LINUX,
            s_rev_level: EXT2_GOOD_OLD_REV,
            s_def_resuid: EXT2_DEF_RESUID,
            s_def_resgid: EXT2_DEF_RESGID,
            s_first_ino: EXT2_GOOD_OLD_FIRST_INO as u32,
            s_inode_size: EXT2_GOOD_OLD_INODE_SIZE as u16,
            s_block_group_nr: block_group_num as u16,
            s_feature_compat: FeatureCompat::from_bits_truncate(0),
            s_feature_incompat: FeatureIncompat::from_bits_truncate(0),
            s_feature_ro_compat: FeatureRocompat::from_bits_truncate(0),
            s_uuid: FAKE_UUID,
            s_volume_name: [0; VOLUMN_NAME_SIZE],
            s_algo_bitmap: 0, // we don't use compression
            s_prealloc_blocks: 0,
            s_last_mounted: [0; MOUNT_SIZE],
            i_padding: [0; 3],
            s_prealloc_dir_blocks: 0,
            p_padding: [0; 2],
            s_journal_uuid: FAKE_JOURNAL_UUID,
            s_journal_inum: 0,
            s_journal_dev: 0,
            s_last_orphan: 0,
            s_hash_seed: [0; HASH_SEED_SIZE],
            s_def_hash_version: 0,
            s_default_mount_option: 0,
            s_first_meta_bg: 0,
            reserved: [0; SB_RESERVED_SIZE]
        };
        sb.s_volume_name[..volumn_name.len()].copy_from_slice(volumn_name.as_bytes());
        sb
    }

    pub fn empty() -> Self {
        Self::new(0, 0, 0, 0, 0,  "Null")
    }

    pub fn check_valid(&self) {
        assert_eq!(self.s_magic, SB_MAGIC, "Bad magic num");
        assert!(self.s_first_data_block == FIRST_DATA_BLOCK as u32, "Wrong first data block");
        assert!(self.s_log_block_size == LOG_BLOCK_SIZE as u32 
                && self.s_log_frag_size == LOG_FRAG_SIZE as u32, 
                "Bad log block size");
        assert!(self.s_blocks_per_group == BLOCKS_PER_GRP as u32 &&
                self.s_frags_per_group == BLOCKS_PER_GRP as u32 &&
                self.s_inodes_per_group == INODES_PER_GRP as u32,
                "Bad inodes and blocks per group");
        assert!(self.s_rev_level == EXT2_GOOD_OLD_REV as u32 &&
                self.s_first_ino == EXT2_GOOD_OLD_FIRST_INO as u32,
                "Bad rev level");
        assert!(self.s_feature_incompat == FeatureIncompat::from_bits_truncate(0),
                "Feature incompat not supported");
        assert!(self.s_feature_ro_compat == FeatureRocompat::from_bits_truncate(0),
                "Feature rocompat not supported");
        assert!(self.s_state == EXT2_VALID_FS,
                "Not a valid state");
    }

}

impl Debug for SuperBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("SuperBlock")
            .field("s_inodes_count", &self.s_inodes_count)
            .field("s_blocks_count", &self.s_blocks_count)
            .field("s_free_inodes_count", &self.s_free_inodes_count)
            .field("s_free_blocks_count", &self.s_free_blocks_count)
            .field("s_mnt_count", &self.s_mnt_count)
            .field("volumn_name", &String::from_utf8_lossy(&self.s_volume_name))
            .finish()
    }
}

impl BlockGroupDesc {
    pub fn new(
        block_bitmap: usize,
        inode_bitmap: usize,
        inode_table: usize,
        free_blocks: usize,
        free_inodes: usize,
        used_dirs: usize,
    ) -> BlockGroupDesc
    {
        BlockGroupDesc {
            bg_block_bitmap: block_bitmap as u32,
            bg_inode_bitmap: inode_bitmap as u32,
            bg_inode_table: inode_table as u32,
            bg_free_blocks_count: free_blocks as u16,
            bg_free_inodes_count: free_inodes as u16,
            bg_used_dirs_count: used_dirs as u16,
            bg_pad: 0,
            bg_reserved: [0; 12]
        }
    }
}

impl DiskInode {
    pub fn new(
        acl_mode: IMODE,
        file_type: u16,
        uid: usize,
        gid: usize,

    ) -> DiskInode
    {
        DiskInode {
            i_mode: acl_mode.bits() | (file_type & 0xF000),
            i_uid: uid as u16,
            i_size: 0,
            i_atime: 0,
            i_mtime: 0,
            i_ctime: 0,
            i_dtime: 0,
            i_gid: gid as u16,
            i_links_count: 1,
            i_blocks: 0,
            i_flags: 0,
            i_osd1: 0,
            i_direct_block: [0; DIRECT_BLOCK_NUM],
            i_double_block: 0,
            i_triple_block: 0,
            i_generation: 0,
            i_dir_acl: 0,
            i_file_acl: 0,
            i_faddr: 0,
            i_osd2: LinuxOSD::empty()
        }
    }

    pub fn acl(&self) -> IMODE {
        IMODE::from_bits_truncate(self.i_mode)
    }

    pub fn file_type(&self) -> u16 {
        self.i_mode & 0xf000
    }

    pub fn file_code(&self) -> u8 {
        match self.file_type() {
            EXT2_S_IFSOCK => EXT2_FT_SOCK,
            EXT2_S_IFLNK => EXT2_FT_SYMLINK,
            EXT2_S_IFREG => EXT2_FT_REG_FILE,
            EXT2_S_IFBLK => EXT2_FT_BLKDEV,
            EXT2_S_IFDIR => EXT2_FT_DIR,
            EXT2_S_IFCHR => EXT2_FT_CHRDEV,
            EXT2_S_IFIFO => EXT2_FT_FIFO,
            _ => EXT2_FT_UNKNOWN
        }
    }

    pub fn is_file(&self) -> bool {
        self.file_type() == EXT2_S_IFREG
    }

    pub fn is_dir(&self) -> bool {
        self.file_type() == EXT2_S_IFDIR
    }

    pub fn data_blocks(&self) -> u32 {
        self.i_blocks * 512 / BLOCK_SIZE as u32
    }

    fn _data_blocks(size: u32) -> u32 {
        (size + BLOCK_SIZE as u32 - 1) / BLOCK_SIZE as u32
    }

    /// Return number of blocks needed include indirect1/2.
    pub fn total_blocks(size: u32) -> u32 {
        let data_blocks = Self::_data_blocks(size) as usize;
        let mut total = data_blocks as usize;
        // indirect1
        if data_blocks > DIRECT_BLOCK_NUM {
            total += 1;
        }
        // indirect2
        if data_blocks > DOUBLE_BLOCK_BOUND {
            total += 1;
            // sub indirect1
            total +=
                (data_blocks - DOUBLE_BLOCK_BOUND + DOUBLE_BLOCK_NUM - 1) / DOUBLE_BLOCK_NUM;
        }
        total as u32
    }

    /// Get the number of data blocks that have to be allocated given the new size of data
    pub fn blocks_num_needed(&self, new_size: u32) -> u32 {
        let allocated_size = 512 * self.i_blocks;
        if new_size <= allocated_size {
            return 0;
        }
        let new_block_num = Self::total_blocks(new_size);
        let old_block_num = Self::total_blocks(allocated_size);
        if old_block_num >= new_block_num {
            0
        } else {
            new_block_num - old_block_num
        }
    }

    /// Get id of block given inner id
    pub fn get_block_id(&self, inner_id: u32, manager: &SpinMutex<BlockCacheManager>) -> u32 {
        debug!("get block id of index {}", inner_id);
        let inner_id = inner_id as usize;
        if inner_id < DIRECT_BLOCK_NUM {
            self.i_direct_block[inner_id]
        } else if inner_id < DOUBLE_BLOCK_BOUND {
            // get_block_cache(self.i_double_block as usize, Arc::clone(block_device))
            //     .lock()
            //     .read(0, |indirect_block: &IndirectBlock| {
            //         indirect_block[inner_id - DIRECT_BLOCK_NUM]
            //     })
            let double_block = manager.lock().get_block_cache(self.i_double_block as _);
            let block_id = double_block.lock()
                .read(0, |indirect_block: &IndirectBlock| {
                            indirect_block[inner_id - DIRECT_BLOCK_NUM]
                });
            block_id
        } else {
            let last = inner_id - DOUBLE_BLOCK_BOUND;
            // let indirect1 = get_block_cache(self.i_triple_block as usize, Arc::clone(block_device))
            //     .lock()
            //     .read(0, |indirect2: &IndirectBlock| {
            //         indirect2[last / DOUBLE_BLOCK_NUM]
            //     });
            let indirect1_block = manager.lock().get_block_cache(self.i_triple_block as _);
            let indirect1 = indirect1_block.lock()
                .read(0, |indirect2: &IndirectBlock| {
                    indirect2[last / DOUBLE_BLOCK_NUM]
                });
            drop(indirect1_block);
            // get_block_cache(indirect1 as usize, Arc::clone(block_device))
            //     .lock()
            //     .read(0, |indirect1: &IndirectBlock| {
            //         indirect1[last % DOUBLE_BLOCK_NUM]
            //     })
            let indirect2_block = manager.lock().get_block_cache(indirect1 as _);
            let block_id = indirect2_block.lock()
                .read(0, |indirect1: &IndirectBlock| {
                    indirect1[last % DOUBLE_BLOCK_NUM]
                });
            block_id
        }
    }

    /// Inncrease the size of current disk inode
    pub fn increase_size(
        &mut self,
        new_size: u32,
        new_blocks: Vec<u32>,
        manager: &SpinMutex<BlockCacheManager>
    ) -> Vec<u32> {
        if new_size <= self.i_size {
            return Vec::new();
        }
        if new_size <= self.i_blocks * 512 {
            self.i_size = new_size;
            return Vec::new();
        }
        
        let mut extra_blocks: Vec<u32> = Vec::new();

        let mut current_blocks = self.data_blocks();
        self.i_size = new_size;
        let mut total_blocks = Self::_data_blocks(new_size);
        self.i_blocks = total_blocks * (BLOCK_SIZE/512) as u32;
        let mut new_blocks = new_blocks.into_iter();
        // fill direct
        while current_blocks < total_blocks.min(DIRECT_BLOCK_NUM as u32) {
            self.i_direct_block[current_blocks as usize] = new_blocks.next().unwrap();
            extra_blocks.push(self.i_direct_block[current_blocks as usize]);
            current_blocks += 1;
        }
        // alloc indirect1
        if total_blocks > DIRECT_BLOCK_NUM as u32 {
            if current_blocks == DIRECT_BLOCK_NUM as u32 {
                self.i_double_block = new_blocks.next().unwrap();
            }
            current_blocks -= DIRECT_BLOCK_NUM as u32;
            total_blocks -= DIRECT_BLOCK_NUM as u32;
        } else {
            return extra_blocks;
        }
        // fill indirect1
        // get_block_cache(self.i_double_block as usize, Arc::clone(block_device))
        //     .lock()
        //     .modify(0, |indirect1: &mut IndirectBlock| {
        //         while current_blocks < total_blocks.min(DOUBLE_BLOCK_NUM as u32) {
        //             indirect1[current_blocks as usize] = new_blocks.next().unwrap();
        //             current_blocks += 1;
        //         }
        //     });
        let double_block = manager.lock().get_block_cache(self.i_double_block as _);
        double_block.lock()
            .modify(0, |indirect1: &mut IndirectBlock| {
                while current_blocks < total_blocks.min(DOUBLE_BLOCK_NUM as u32) {
                    indirect1[current_blocks as usize] = new_blocks.next().unwrap();
                    extra_blocks.push(indirect1[current_blocks as usize]);
                    current_blocks += 1;
                }
            });
        manager.lock().release_block(double_block);
        // alloc indirect2
        if total_blocks > DOUBLE_BLOCK_NUM as u32 {
            if current_blocks == DOUBLE_BLOCK_NUM as u32 {
                self.i_triple_block = new_blocks.next().unwrap();
            }
            current_blocks -= DOUBLE_BLOCK_NUM as u32;
            total_blocks -= DOUBLE_BLOCK_NUM as u32;
        } else {
            return extra_blocks;
        }
        // fill indirect2 from (a0, b0) -> (a1, b1)
        let a0 = current_blocks as usize / DOUBLE_BLOCK_NUM;
        let b0 = current_blocks as usize % DOUBLE_BLOCK_NUM;
        let a1 = total_blocks as usize / DOUBLE_BLOCK_NUM;
        let b1 = total_blocks as usize % DOUBLE_BLOCK_NUM;
        // alloc low-level indirect1
        let indirect1_block = manager.lock().get_block_cache(self.i_triple_block as _);
        indirect1_block.lock()
            .modify(0, |indirect1: &mut IndirectBlock| {
                for a in a0..=a1 {
                    // if b0 == 0 {
                    //     indirect2[a0] = new_blocks.next().unwrap();
                    // }
                    // // fill current
                    // get_block_cache(indirect2[a0] as usize, Arc::clone(block_device))
                    //     .lock()
                    //     .modify(0, |indirect1: &mut IndirectBlock| {
                    //         indirect1[b0] = new_blocks.next().unwrap();
                    //     });
                    // // move to next
                    // b0 += 1;
                    // if b0 == DOUBLE_BLOCK_NUM {
                    //     b0 = 0;
                    //     a0 += 1;
                    // }
                    let start = if a == a0 { b0 } else { 0 };
                    let end = if a == a1 { b1 } else { DOUBLE_BLOCK_NUM };
                    if start == 0 && end > 0 {
                        indirect1[a] = new_blocks.next().unwrap();
                    }
                    let indirect2_block = manager.lock().get_block_cache(indirect1[a] as _);
                    indirect2_block.lock()
                        .modify(0, |indirect2: &mut IndirectBlock| {
                            for b in start..end {
                                indirect2[b] = new_blocks.next().unwrap();
                                extra_blocks.push(indirect2[b]);
                            }
                        });
                    manager.lock().release_block(indirect2_block);

                }
            });
        manager.lock().release_block(indirect1_block);
        return extra_blocks;
    }

    /// Clear size to zero and return blocks that should be deallocated.
    /// We will clear the block contents to zero later.
    pub fn clear_size(&mut self, manager: &SpinMutex<BlockCacheManager>) -> Vec<u32> {
        self.decrease_size(0, manager)
    }

    /// Get all data blocks of current inode
    pub fn all_data_blocks(&self, manager: &SpinMutex<BlockCacheManager>, include_index: bool) -> Vec<u32> {
        let mut v: Vec<u32> = Vec::new();
        let mut data_blocks = self.data_blocks() as usize;
        // debug!("all_data_blocks called on {} blocks", data_blocks);
        // self.size = 0;
        let mut current_blocks = 0usize;
        // direct
        while current_blocks < data_blocks.min(DIRECT_BLOCK_NUM) {
            v.push(self.i_direct_block[current_blocks]);
            // self.direct[current_blocks] = 0;
            current_blocks += 1;
        }
        // debug!("after direct: {}", v.len());
        // indirect1 block
        if data_blocks > DIRECT_BLOCK_NUM {
            if include_index {
                v.push(self.i_double_block);
            }
            data_blocks -= DIRECT_BLOCK_NUM;
            current_blocks = 0;
        } else {
            return v;
        }
        // indirect1
        let double_block = manager.lock().get_block_cache(self.i_double_block as _);
        double_block.lock()
            .read(0, |indirect1: &IndirectBlock| {
                while current_blocks < data_blocks.min(DOUBLE_BLOCK_NUM) {
                    v.push(indirect1[current_blocks]);
                    current_blocks += 1;
                }
            });
        manager.lock().release_block(double_block);
        // debug!("after double block: {}", v.len());
        // indirect2 block
        if data_blocks > DOUBLE_BLOCK_NUM {
            if include_index {
                v.push(self.i_triple_block);
            }
            data_blocks -= DOUBLE_BLOCK_NUM;
        } else {
            return v;
        }
        // indirect2
        assert!(data_blocks <= TRIPLE_BLOCK_NUM);
        let a1 = data_blocks / DOUBLE_BLOCK_NUM;
        let b1 = data_blocks % DOUBLE_BLOCK_NUM;
        let indirect1_block = manager.lock().get_block_cache(self.i_triple_block as _);
        indirect1_block.lock()
            .read(0, |indirect2: &IndirectBlock| {
                // full indirect1 blocks
                for entry in indirect2.iter().take(a1) {
                    if include_index {
                        v.push(*entry);
                    }
                    let indirect2_block = manager.lock().get_block_cache(*entry as _);
                    indirect2_block.lock()
                        .read(0, |indirect1: &IndirectBlock| {
                            for entry in indirect1.iter() {
                                v.push(*entry);
                            }
                        });
                    manager.lock().release_block(indirect2_block);
                }
                // last indirect1 block
                if b1 > 0 {
                    if include_index {
                        v.push(indirect2[a1]);
                    }
                    let indirect2_block = manager.lock().get_block_cache(indirect2[a1] as _);
                    indirect2_block.lock()
                        .read(0, |indirect1: &IndirectBlock| {
                            for entry in indirect1.iter().take(b1) {
                                v.push(*entry);
                            }
                        });
                    manager.lock().release_block(indirect2_block);
                }
            });
        manager.lock().release_block(indirect1_block);
        v
    }

    /// Decrease size
    pub fn decrease_size(&mut self, new_size: u32, manager: &SpinMutex<BlockCacheManager>) -> Vec<u32> {
        // debug!("decrease size from {} to {}", self.i_size, new_size);
        if new_size >= self.i_size {
            return Vec::new();
        }
        if Self::total_blocks(new_size) >= Self::total_blocks(self.i_blocks * 512) {
            self.i_size = new_size;
            return Vec::new();
        }
        let mut all_blocks = self.all_data_blocks(manager, true);
        self.i_size = new_size;
        self.i_blocks = Self::_data_blocks(new_size) * BLOCK_SIZE as u32 / 512;
        let remain_block_num = Self::total_blocks(new_size);
        all_blocks.drain(0..remain_block_num as usize);
        
        all_blocks
    }

    /// Read data from current disk inode
    pub fn read_at(
        &self,
        offset: usize,
        buf: &mut [u8],
        manager: &SpinMutex<BlockCacheManager>,
        cache: Option<&Vec<u32>>
    ) -> usize {
        let mut start = offset;
        let end = (offset + buf.len()).min(self.i_size as usize);
        if start >= end {
            return 0;
        }
        let mut start_block = start / BLOCK_SIZE;
        let mut read_size = 0usize;
        loop {
            // calculate end of current block
            let mut end_current_block = (start / BLOCK_SIZE + 1) * BLOCK_SIZE;
            end_current_block = end_current_block.min(end);
            // read and update read size
            let block_read_size = end_current_block - start;
            let dst = &mut buf[read_size..read_size + block_read_size];
            let block_id = if let Some(blocks) = cache.as_ref() {
                blocks[start_block]
            } else {
                self.get_block_id(start_block as _, manager)
            };
            let data_block = manager.lock().get_block_cache(block_id as _);
            data_block.lock()
            .read(0, |data_block: &DataBlock| {
                let src = &data_block[start % BLOCK_SIZE..start % BLOCK_SIZE + block_read_size];
                dst.copy_from_slice(src);
            });
            manager.lock().release_block(data_block);
            read_size += block_read_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_block += 1;
            start = end_current_block;
        }
        read_size
    }
    /// Write data into current disk inode
    /// size must be adjusted properly beforehand
    pub fn write_at(
        &mut self,
        offset: usize,
        buf: &[u8],
        manager: &SpinMutex<BlockCacheManager>,
        cache: Option<&Vec<u32>>
    ) -> usize {
        let mut start = offset;
        let end = (offset + buf.len()).min(self.i_size as usize);
        assert!(start <= end);
        let mut start_block = start / BLOCK_SIZE;
        let mut write_size = 0usize;
        loop {
            // calculate end of current block
            let mut end_current_block = (start / BLOCK_SIZE + 1) * BLOCK_SIZE;
            end_current_block = end_current_block.min(end);
            // write and update write size
            let block_write_size = end_current_block - start;
            let block_id = if let Some(blocks) = cache.as_ref() {
                blocks[start_block]
            } else {
                self.get_block_id(start_block as _, manager)
            };
            let data_block = manager.lock().get_block_cache(block_id as _);
            data_block.lock()
            .modify(0, |data_block: &mut DataBlock| {
                let src = &buf[write_size..write_size + block_write_size];
                let dst = &mut data_block[start % BLOCK_SIZE..start % BLOCK_SIZE + block_write_size];
                dst.copy_from_slice(src);
            });
            manager.lock().release_block(data_block);
            write_size += block_write_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_block += 1;
            start = end_current_block;
        }
        write_size
    }
}


impl DirEntryHead {
    pub fn create(inode: usize, name: &str, file_type: u8) -> DirEntryHead {
        let name_len = name.as_bytes().len().min(MAX_NAME_LEN);
        let rec_len = size_of::<DirEntryHead>() + name_len;

        DirEntryHead {
            inode: inode as u32,
            rec_len: rec_len as u16,
            name_len: name_len as u8,
            file_type
        }
    }

    pub fn empty() -> Self {
        DirEntryHead { inode: 0, rec_len: 0, name_len: 0, file_type: 0 }
    }

    /// Serialize into bytes
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self as *const _ as usize as *const u8, size_of::<DirEntryHead>()) }
    }
    /// Serialize into mutable bytes
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as usize as *mut u8, size_of::<DirEntryHead>()) }
    }
}

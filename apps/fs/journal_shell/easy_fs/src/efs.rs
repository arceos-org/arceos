#[cfg(feature = "journal")]
use core::cell::RefCell;

use super::{
    block_cache_sync_all, get_block_cache, Bitmap, BlockDevice, DiskInode, DiskInodeType, Inode,
    SuperBlock,
};
#[cfg(feature = "journal")]
use crate::journal::{get_buffer_dyn, SystemProvider};
use crate::{config::JOURNAL_SIZE, BLOCK_SZ};
use alloc::{rc::Rc, sync::Arc};
#[cfg(feature = "journal")]
use axfs_vfs::VfsError;
use axfs_vfs::VfsOps;
#[cfg(feature = "journal")]
use axfs_vfs::VfsResult;
use spin::Mutex;

///An easy file system on block
pub struct EasyFileSystem {
    /// Real device
    pub block_device: Rc<dyn BlockDevice>,
    /// Inode bitmap
    pub inode_bitmap: Bitmap,
    /// Data bitmap
    pub data_bitmap: Bitmap,
    inode_area_start_block: u32,
    data_area_start_block: u32,
    #[allow(unused)]
    journal_start_block: u32,
    #[allow(unused)]
    journal_size: u32,
    ///
    #[cfg(feature = "journal")]
    pub journal: Rc<RefCell<jbd::Journal>>,
}

type DataBlock = [u8; BLOCK_SZ];
/// An easy fs over a block device
impl EasyFileSystem {
    /// A data block of block size
    pub fn create(
        block_device: Rc<dyn BlockDevice>,
        total_blocks: u32,
        inode_bitmap_blocks: u32,
    ) -> Self {
        let total_blocks = total_blocks - JOURNAL_SIZE;
        // calculate block size of areas & create bitmaps
        let inode_bitmap = Bitmap::new(1, inode_bitmap_blocks as usize);
        let inode_num = inode_bitmap.maximum();
        let inode_area_blocks =
            ((inode_num * core::mem::size_of::<DiskInode>() + BLOCK_SZ - 1) / BLOCK_SZ) as u32;
        let inode_total_blocks = inode_bitmap_blocks + inode_area_blocks;
        let data_total_blocks = total_blocks - 1 - inode_total_blocks;
        let data_bitmap_blocks = (data_total_blocks + 4096) / 4097;
        let data_area_blocks = data_total_blocks - data_bitmap_blocks;
        let data_bitmap = Bitmap::new(
            (1 + inode_bitmap_blocks + inode_area_blocks) as usize,
            data_bitmap_blocks as usize,
        );
        #[cfg(feature = "journal")]
        let provider = Rc::new(SystemProvider::new());

        #[cfg(feature = "journal")]
        let mut journal = jbd::Journal::init_dev(
            provider,
            block_device.clone(),
            block_device.clone(),
            total_blocks,
            JOURNAL_SIZE,
        )
        .unwrap();

        #[cfg(feature = "journal")]
        journal.create().unwrap();

        let mut efs = Self {
            block_device: Rc::clone(&block_device),
            inode_bitmap,
            data_bitmap,
            inode_area_start_block: 1 + inode_bitmap_blocks,
            data_area_start_block: 1 + inode_total_blocks + data_bitmap_blocks,
            journal_start_block: total_blocks,
            journal_size: JOURNAL_SIZE,
            #[cfg(feature = "journal")]
            journal: Rc::new(RefCell::new(journal)),
        };
        // clear all blocks
        for i in 0..total_blocks {
            get_block_cache(i as usize, Rc::clone(&block_device)).modify(
                0,
                |data_block: &mut DataBlock| {
                    for byte in data_block.iter_mut() {
                        *byte = 0;
                    }
                },
            );
        }
        // initialize SuperBlock
        get_block_cache(0, Rc::clone(&block_device)).modify(0, |super_block: &mut SuperBlock| {
            super_block.initialize(
                total_blocks,
                inode_bitmap_blocks,
                inode_area_blocks,
                data_bitmap_blocks,
                data_area_blocks,
                total_blocks,
                JOURNAL_SIZE,
            );
        });
        // write back immediately
        // create a inode for root node "/"
        assert_eq!(
            efs.alloc_inode(
                #[cfg(feature = "journal")]
                None
            ),
            0
        );
        let (root_inode_block_id, root_inode_offset) = efs.get_disk_inode_pos(0);
        get_block_cache(root_inode_block_id as usize, Rc::clone(&block_device)).modify(
            root_inode_offset,
            |disk_inode: &mut DiskInode| {
                disk_inode.initialize(DiskInodeType::Directory);
            },
        );
        block_cache_sync_all();

        efs
    }

    /// Open a block device as a filesystem
    pub fn open(block_device: Rc<dyn BlockDevice>) -> Self {
        // read SuperBlock
        get_block_cache(0, Rc::clone(&block_device)).read(0, |super_block: &SuperBlock| {
            assert!(super_block.is_valid(), "Error loading EFS!");
            let inode_total_blocks =
                super_block.inode_bitmap_blocks + super_block.inode_area_blocks;

            #[cfg(feature = "journal")]
            let provider = Rc::new(SystemProvider::new());

            #[cfg(feature = "journal")]
            let mut journal = jbd::Journal::init_dev(
                provider,
                block_device.clone(),
                block_device.clone(),
                super_block.journal_start,
                super_block.journal_len,
            )
            .unwrap();

            #[cfg(feature = "journal")]
            journal.load().unwrap();

            Self {
                block_device,
                inode_bitmap: Bitmap::new(1, super_block.inode_bitmap_blocks as usize),
                data_bitmap: Bitmap::new(
                    (1 + inode_total_blocks) as usize,
                    super_block.data_bitmap_blocks as usize,
                ),
                inode_area_start_block: 1 + super_block.inode_bitmap_blocks,
                data_area_start_block: 1 + inode_total_blocks + super_block.data_bitmap_blocks,
                journal_start_block: super_block.journal_start,
                journal_size: super_block.journal_len,
                #[cfg(feature = "journal")]
                journal: Rc::new(RefCell::new(journal)),
            }
        })
    }

    /// Reload the filesystem
    pub fn load(&mut self, block_device: Rc<dyn BlockDevice>) {
        *self = Self::open(block_device);
    }
    /// Get the root inode of the filesystem
    pub fn root_inode(efs: &Arc<Mutex<Self>>) -> Inode {
        let efs_lock = efs.lock();
        let block_device = Rc::clone(&efs_lock.block_device);
        // acquire efs lock temporarily
        let (block_id, block_offset) = efs_lock.get_disk_inode_pos(0);
        // release efs lock
        Inode::new(0, block_id, block_offset, Arc::clone(efs), block_device)
    }
    /// Get inode by id
    pub fn get_disk_inode_pos(&self, inode_id: u32) -> (u32, usize) {
        let inode_size = core::mem::size_of::<DiskInode>();
        let inodes_per_block = (BLOCK_SZ / inode_size) as u32;
        let block_id = self.inode_area_start_block + inode_id / inodes_per_block;
        (
            block_id,
            (inode_id % inodes_per_block) as usize * inode_size,
        )
    }
    /// Get data block by id
    pub fn get_data_block_id(&self, data_block_id: u32) -> u32 {
        self.data_area_start_block + data_block_id
    }
    /// Allocate a new inode
    pub fn alloc_inode(
        &mut self,
        #[cfg(feature = "journal")] handle: Option<&mut jbd::Handle>,
    ) -> u32 {
        #[allow(unused)]
        let (dblock, bitmap_block) = self.inode_bitmap.alloc(&self.block_device).unwrap();
        #[cfg(feature = "journal")]
        if let Some(handle) = handle {
            let buf = get_buffer_dyn(&self.block_device, bitmap_block).unwrap();
            handle.get_write_access(&buf).unwrap();
            handle.dirty_metadata(&buf).unwrap();
        }
        dblock as u32
    }

    /// Allocate a data block
    pub fn alloc_data(
        &mut self,
        #[cfg(feature = "journal")] handle: Option<&mut jbd::Handle>,
    ) -> u32 {
        #[allow(unused)]
        let (dblock, bitmap_block) = self.data_bitmap.alloc(&self.block_device).unwrap();
        #[cfg(feature = "journal")]
        if let Some(handle) = handle {
            let buf = get_buffer_dyn(&self.block_device, bitmap_block).unwrap();
            // FIXME: We should use get_undo_access here and use along with the `commit_data`
            // of the journal buffer to actually determine the allocatable blocks.
            handle.get_write_access(&buf).unwrap();
            handle.dirty_metadata(&buf).unwrap();
        }
        dblock as u32 + self.data_area_start_block
    }
    /// Deallocate a data block
    pub fn dealloc_data(
        &mut self,
        block_id: u32,
        #[cfg(feature = "journal")] handle: &mut jbd::Handle,
    ) {
        get_block_cache(block_id as usize, Rc::clone(&self.block_device)).modify(
            0,
            |data_block: &mut DataBlock| {
                data_block.iter_mut().for_each(|p| {
                    *p = 0;
                })
            },
        );

        #[allow(unused)]
        let bitmap_block_id = self.data_bitmap.dealloc(
            &self.block_device,
            (block_id - self.data_area_start_block) as usize,
        );

        #[cfg(feature = "journal")]
        let buf = get_buffer_dyn(&self.block_device, bitmap_block_id).unwrap();
        // FIXME: We should use get_undo_access here instead.
        #[cfg(feature = "journal")]
        handle.get_write_access(&buf).unwrap();
        #[cfg(feature = "journal")]
        handle.dirty_metadata(&buf).unwrap();
    }

    #[cfg(feature = "journal")]
    pub(crate) fn journal_start(&self, nblocks: u32) -> VfsResult<Rc<RefCell<jbd::Handle>>> {
        jbd::Journal::start(self.journal.clone(), nblocks).map_err(|_| VfsError::Journal)
    }

    #[cfg(feature = "journal")]
    pub(crate) fn journal_commit(&self) {
        let mut journal = self.journal.as_ref().borrow_mut();
        journal.commit_transaction().unwrap();
    }

    /// Do checkpoint
    #[cfg(feature = "journal")]
    pub fn journal_checkpoint(&self) {
        let mut journal = self.journal.as_ref().borrow_mut();
        journal.do_all_checkpoints();
    }
}

///
pub struct EasyFileSystemWrapper(Arc<Mutex<EasyFileSystem>>);

unsafe impl Send for EasyFileSystemWrapper {}
unsafe impl Sync for EasyFileSystemWrapper {}

impl EasyFileSystemWrapper {
    pub fn inner(&self) -> Arc<Mutex<EasyFileSystem>> {
        self.0.clone()
    }
}

impl VfsOps for EasyFileSystemWrapper {
    fn format(&self) -> axfs_vfs::VfsResult {
        Ok(())
    }
    fn mount(&self, _path: &str, _mount_point: axfs_vfs::VfsNodeRef) -> axfs_vfs::VfsResult {
        Ok(())
    }
    fn root_dir(&self) -> axfs_vfs::VfsNodeRef {
        Arc::new(EasyFileSystem::root_inode(&self.0))
    }
    fn statfs(&self) -> axfs_vfs::VfsResult<axfs_vfs::FileSystemInfo> {
        todo!()
    }
    fn umount(&self) -> axfs_vfs::VfsResult {
        #[cfg(feature = "journal")]
        return self
            .0
            .lock()
            .journal
            .as_ref()
            .borrow_mut()
            .destroy()
            .map_err(|_| VfsError::Journal);

        #[cfg(not(feature = "journal"))]
        Ok(())
    }
}

impl EasyFileSystemWrapper {
    ///
    pub fn new(efs: EasyFileSystem) -> Self {
        Self(Arc::new(Mutex::new(efs)))
    }
}

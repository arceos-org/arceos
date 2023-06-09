#[cfg(feature = "journal")]
use alloc::boxed::Box;
#[cfg(feature = "journal")]
use core::any::Any;
use core::cell::{RefCell, UnsafeCell};
#[cfg(feature = "journal")]
use jbd::sal::Buffer;

use super::{BlockDevice, BLOCK_SZ};
use alloc::collections::VecDeque;
use alloc::rc::Rc;

struct BlockCacheInner {
    /// cached block data
    cache: [u8; BLOCK_SZ],
    /// underlying block id
    block_id: usize,
    /// underlying block device
    block_device: Rc<dyn BlockDevice>,
    /// whether the block is dirty
    modified: bool,
    #[cfg(feature = "journal")]
    jbd_dirty: bool,
    #[cfg(feature = "journal")]
    private: Option<Box<dyn Any>>,
    #[cfg(feature = "journal")]
    jbd_managed: bool,
    #[cfg(feature = "journal")]
    revoked: bool,
    #[cfg(feature = "journal")]
    revoke_valid: bool,
}

/// Cached block inside memory
pub struct BlockCache(RefCell<BlockCacheInner>);

impl BlockCache {
    /// Load a new BlockCache from disk.
    pub fn new(block_id: usize, block_device: Rc<dyn BlockDevice>) -> Self {
        let mut cache = [0u8; BLOCK_SZ];
        block_device.read_block(block_id, &mut cache);
        Self(RefCell::new(BlockCacheInner {
            cache,
            block_id,
            block_device,
            modified: false,
            #[cfg(feature = "journal")]
            jbd_dirty: false,
            #[cfg(feature = "journal")]
            private: None,
            #[cfg(feature = "journal")]
            jbd_managed: false,
            #[cfg(feature = "journal")]
            revoked: false,
            #[cfg(feature = "journal")]
            revoke_valid: false,
        }))
    }
    /// Get the address of an offset inside the cached block data
    fn addr_of_offset(&self, offset: usize) -> usize {
        &self.0.borrow().cache[offset] as *const _ as usize
    }

    pub fn get_ref<T>(&self, offset: usize) -> &T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SZ);
        let addr = self.addr_of_offset(offset);
        unsafe { &*(addr as *const T) }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn get_mut<T>(&self, offset: usize) -> &mut T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SZ);
        self.0.borrow_mut().modified = true;
        let addr = self.addr_of_offset(offset);
        unsafe { &mut *(addr as *mut T) }
    }

    pub fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
        f(self.get_ref(offset))
    }

    pub fn modify<T, V>(&self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V {
        f(self.get_mut(offset))
    }

    #[cfg(not(feature = "journal"))]
    fn sync(&self) {
        let mut inner = self.0.borrow_mut();
        if inner.modified {
            inner.modified = false;
            inner.block_device.write_block(inner.block_id, &inner.cache);
        }
    }
}

#[cfg(feature = "journal")]
impl jbd::sal::Buffer for BlockCache {
    fn block_id(&self) -> usize {
        self.0.borrow().block_id
    }
    fn size(&self) -> usize {
        BLOCK_SZ
    }
    fn dirty(&self) -> bool {
        self.0.borrow().modified
    }
    fn mark_dirty(&self) {
        self.0.borrow_mut().modified = true;
    }
    fn clear_dirty(&self) {
        self.0.borrow_mut().modified = false;
    }
    fn jbd_dirty(&self) -> bool {
        self.0.borrow().jbd_dirty
    }
    fn mark_jbd_dirty(&self) {
        self.0.borrow_mut().jbd_dirty = true;
    }
    fn clear_jbd_dirty(&self) {
        self.0.borrow_mut().jbd_dirty = false;
    }
    fn data(&self) -> *mut u8 {
        self.0.borrow().cache.as_ptr() as *mut u8
    }
    fn private(&self) -> &Option<Box<dyn Any>> {
        unsafe { &*(&self.0.borrow().private as *const _) }
    }
    fn set_private(&self, private: Option<Box<dyn Any>>) {
        self.0.borrow_mut().private = private;
    }
    fn jbd_managed(&self) -> bool {
        self.0.borrow().jbd_managed
    }
    fn set_jbd_managed(&self, managed: bool) {
        self.0.borrow_mut().jbd_managed = managed;
    }
    fn revoked(&self) -> bool {
        self.0.borrow().revoked
    }
    fn set_revoked(&self) {
        self.0.borrow_mut().revoked = true;
    }
    fn clear_revoked(&self) {
        self.0.borrow_mut().revoked = false;
    }
    fn revoke_valid(&self) -> bool {
        self.0.borrow().revoke_valid
    }
    fn set_revoke_valid(&self) {
        self.0.borrow_mut().revoke_valid = true;
    }
    fn clear_revoke_valid(&self) {
        self.0.borrow_mut().revoke_valid = false;
    }
    fn test_clear_dirty(&self) -> bool {
        let mut inner = self.0.borrow_mut();
        let ret = inner.modified;
        inner.modified = false;
        ret
    }
    fn test_clear_jbd_dirty(&self) -> bool {
        let mut inner = self.0.borrow_mut();
        let ret = inner.jbd_dirty;
        inner.jbd_dirty = false;
        ret
    }
    fn test_clear_revoke_valid(&self) -> bool {
        let mut inner = self.0.borrow_mut();
        let ret = inner.revoke_valid;
        inner.revoke_valid = false;
        ret
    }
    fn test_clear_revoked(&self) -> bool {
        let mut inner = self.0.borrow_mut();
        let ret = inner.revoked;
        inner.revoked = false;
        ret
    }
    fn test_set_revoke_valid(&self) -> bool {
        let mut inner = self.0.borrow_mut();
        let ret = inner.revoke_valid;
        inner.revoke_valid = true;
        ret
    }
    fn test_set_revoked(&self) -> bool {
        let mut inner = self.0.borrow_mut();
        let ret = inner.revoked;
        inner.revoked = true;
        ret
    }
    fn sync(&self) {
        let mut inner = self.0.borrow_mut();
        if inner.modified {
            inner.modified = false;
            inner.block_device.write_block(inner.block_id, &inner.cache);
        }
    }
}

impl Drop for BlockCache {
    fn drop(&mut self) {
        self.sync();
    }
}

/// Use a block cache of 16 blocks
const BLOCK_CACHE_SIZE: usize = 512;

pub struct BlockCacheManager {
    pub queue: VecDeque<(usize, Rc<BlockCache>)>,
}

impl BlockCacheManager {
    pub const fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn get_block_cache(
        &mut self,
        block_id: usize,
        block_device: Rc<dyn BlockDevice>,
    ) -> Rc<BlockCache> {
        if let Some(pair) = self.queue.iter().find(|pair| pair.0 == block_id) {
            Rc::clone(&pair.1)
        } else {
            // substitute
            if self.queue.len() == BLOCK_CACHE_SIZE {
                // from front to tail
                if let Some((idx, _)) = self
                    .queue
                    .iter()
                    .enumerate()
                    .find(|(_, pair)| Rc::strong_count(&pair.1) == 1)
                {
                    self.queue.drain(idx..=idx);
                } else {
                    panic!("Run out of BlockCache!");
                }
            }
            // load block into mem and push back
            let block_cache = Rc::new(BlockCache::new(block_id, Rc::clone(&block_device)));
            self.queue.push_back((block_id, Rc::clone(&block_cache)));
            block_cache
        }
    }
}

struct SyncCell<T>(UnsafeCell<T>);
unsafe impl<T> Sync for SyncCell<T> {}
impl<T> SyncCell<T> {
    const fn new(v: T) -> Self {
        Self(UnsafeCell::new(v))
    }
    #[allow(clippy::mut_from_ref)]
    unsafe fn get_mut(&self) -> &mut T {
        &mut *self.0.get()
    }
}
impl<T: Copy> SyncCell<T> {}
unsafe impl Send for BlockCacheManager {}

static BLOCK_CACHE_MANAGER: SyncCell<BlockCacheManager> = SyncCell::new(BlockCacheManager::new());

pub fn block_cache_manager() -> &'static mut BlockCacheManager {
    unsafe { BLOCK_CACHE_MANAGER.get_mut() }
}

/// Get the block cache corresponding to the given block id and block device
pub fn get_block_cache(block_id: usize, block_device: Rc<dyn BlockDevice>) -> Rc<BlockCache> {
    block_cache_manager().get_block_cache(block_id, block_device)
}
/// Sync all block cache to block device
pub fn block_cache_sync_all() {
    for (_, cache) in block_cache_manager().queue.iter() {
        cache.sync();
    }
}

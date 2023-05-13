#![allow(unused)]
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::collections::BTreeMap;
use fs_utils::{InListNode, ListNode, inlist_access};
use core::marker::PhantomData;
use core::ops::DerefMut;
use crate::mutex::SpinMutex;
use crate::block_dev::{BlockDevice, NullDevice};
use crate::config::BLOCK_SIZE;
use log::*;

inlist_access!(pub ManagerAccessBlockCache, BlockCache, lru_head);

pub struct BlockCache {
    lru_head: InListNode<BlockCache, ManagerAccessBlockCache>,
    block_id: usize,
    modified: bool,
    valid: bool,
    cache: Box<[u8]>
}

impl BlockCache {
    pub fn new(block_id: usize, block_size: usize) -> Option<Self> {
        match unsafe { Box::try_new_uninit_slice(block_size) } {
            Ok(cache) => Some(Self {
                lru_head: InListNode::new(),
                block_id,
                modified: false,
                valid: false,
                cache: unsafe {cache.assume_init()}
            }),
            Err(_) => None
        }
    }

    /// Get the address of an offset inside the cached block data
    fn addr_of_offset(&self, offset: usize) -> usize {
        &self.cache[offset] as *const _ as usize
    }

    pub fn get_ref<T>(&self, offset: usize) -> &T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= self.cache.len());
        let addr = self.addr_of_offset(offset);
        unsafe { &*(addr as *const T) }
    }

    pub fn get_mut<T>(&mut self, offset: usize) -> &mut T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= self.cache.len());
        self.modified = true;
        let addr = self.addr_of_offset(offset);
        unsafe { &mut *(addr as *mut T) }
    }

    pub fn zero(&mut self) {
        self.modified = true;
        for byte in self.cache.as_mut() {
            *byte = 0;
        }
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    pub fn read<T: Sized, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
        f(self.get_ref(offset))
    }

    pub fn modify<T: Sized, V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V {
        f(self.get_mut(offset))
    }
}

pub struct BlockCacheManager {
    device: Arc<dyn BlockDevice>,
    max_cache: usize,
    blocks: BTreeMap<usize,Arc<SpinMutex<BlockCache>>>,
    lru_head: InListNode<BlockCache, ManagerAccessBlockCache>
}

impl BlockCacheManager {
    pub fn new() -> Self {
        Self {
            device: Arc::new(NullDevice),
            max_cache: 0,
            blocks: BTreeMap::new(),
            lru_head: InListNode::new()
        }
    }

    pub fn init(&mut self, block_device: Arc<dyn BlockDevice>, max_cache: usize) {
        // TODO: modify bitmap to adapt to variant length block
        assert!(block_device.block_size() == BLOCK_SIZE);
        self.device = block_device;
        self.max_cache = max_cache;
        self.blocks.clear();
        self.lru_head.lazy_init();
    }

    pub fn get_block_cache(&mut self, block_id: usize) -> Arc<SpinMutex<BlockCache>> {
        // debug!("get_block_cache {}", block_id);
        if let Some(cache) = self.blocks.get(&block_id) {
            return cache.clone();
        }
        
        if self.blocks.len() < self.max_cache {
            let mut new_cache = BlockCache::new(block_id, self.device.block_size()).unwrap();
            // init
            self.device.read_block(block_id, new_cache.cache.as_mut());
            new_cache.valid = true;
            let new_cache = Arc::new(SpinMutex::new(new_cache));
            new_cache.lock().lru_head.lazy_init();
            // debug!("&self.lru_head = {}", &self.lru_head as *const _ as usize);
            // self.lru_head.list_check();
            self.lru_head.push_prev(unsafe {&mut new_cache.unsafe_get_mut().lru_head});
            self.blocks.insert(block_id, new_cache.clone());
            // self.lru_head.list_check();
            return new_cache;
        };

        // evict a block
        for bk in self.lru_head.next_iter() {
            if Arc::strong_count(self.blocks.get(&bk.block_id).unwrap()) == 1 {
                let evict_cache = self.blocks.remove(&bk.block_id).unwrap();
                // write dirty data to disk
                self.write_block(&evict_cache);

                let cache_ref = unsafe { evict_cache.unsafe_get_mut() };
                // unsafe {(*ptr).lru_head.pop_self();}

                // init evicted block
                self.device.read_block(block_id, &mut cache_ref.cache);
                // self.lru_head.list_check();
                cache_ref.modified = false;
                cache_ref.valid = true;
                cache_ref.block_id = block_id;

                // insert to block map
                self.blocks.insert(block_id, evict_cache.clone());
                // self.lru_head.list_check();
                return evict_cache;
            }
        }

        panic!("Run out of blocks");
    }

    /// Safety
    /// 
    /// Should drop lock of BlockCache right before calling this function to avoid dead lock
    pub fn release_block(&mut self, bac: Arc<SpinMutex<BlockCache>>) {
        if Arc::strong_count(&bac) == 2 {
            let ptr = unsafe { bac.unsafe_get_mut() };
            ptr.lru_head.pop_self();
            self.lru_head.push_prev(&mut ptr.lru_head);
            
        }
    }

    pub fn write_block(&self, block: &Arc<SpinMutex<BlockCache>>) {
        let mut lk = block.lock();
        if lk.modified {
            lk.modified = false;
            self.device.write_block(lk.block_id, lk.cache.as_ref());
        }
    }

    /// Use arc to record refcnt
    /// when unpin, just drop(cache)
    pub fn pin_block(&self, block_id: usize) -> Option<Arc<SpinMutex<BlockCache>>> {
        self.blocks.get(&block_id).map(|ac| ac.clone())
    }

    /// Move arc to this function, it will be dropped right away
    pub fn unpin_block(&self, bac: Arc<SpinMutex<BlockCache>>) {  }

    /// Write all dirty blocks to disk
    pub fn sync_all_block(&self) {
        debug!("sync all blocks");
        for (_, block) in self.blocks.iter() {
            let mut lk = block.lock();
            if lk.modified {
                lk.modified = false;
                debug!("Write to block {}", lk.block_id);
                self.device.write_block(lk.block_id, lk.cache.as_ref());
            }
        }
    }
}

impl Drop for BlockCacheManager {
    fn drop(&mut self) {
        self.sync_all_block();
    }
}
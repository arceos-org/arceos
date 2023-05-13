use crate::vfs::InodeCache;
use crate::efs::Ext2FileSystem;
use crate::mutex::SpinMutex;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;


pub struct InodeCacheManager {
    inodes: BTreeMap<usize, Arc<SpinMutex<InodeCache>>>,
    max_inode: usize
}

impl InodeCacheManager {
    pub fn new(max_inode: usize) -> InodeCacheManager {
        Self { inodes: BTreeMap::new(), max_inode }
    }

    pub fn get_or_insert(&mut self, inode_id: usize, fs: &Arc<Ext2FileSystem>) -> Option<Arc<SpinMutex<InodeCache>>> {
        if let Some(inode_cache) = self.inodes.get(&inode_id).map(|cache| cache.clone()) {
            // in cache
            Some(inode_cache)
        } else {
            if self.inodes.len() < self.max_inode {
                // insert directly
                if let Some(cache) = Ext2FileSystem::create_inode_cache(fs, inode_id) {
                    let inode_cache = Arc::new(SpinMutex::new(cache));
                    self.inodes.insert(inode_id, inode_cache.clone());
                    Some(inode_cache)
                } else {
                    None
                }
            } else {
                // first find an inode_cache to evict
                if let Some(evict_inode_id) 
                    = self.inodes.iter()
                    .find(|(_, cache)| Arc::strong_count(cache) == 1)
                    .map(|(id, _)| *id)
                {
                    self.inodes.remove(&evict_inode_id);
                    if let Some(cache) = Ext2FileSystem::create_inode_cache(fs, inode_id) {
                        let inode_cache = Arc::new(SpinMutex::new(cache));
                        self.inodes.insert(inode_id, inode_cache.clone());
                        Some(inode_cache)
                    } else {
                        None
                    }
                } else {
                    panic!("No free inode");
                }
            }
        }
    }

    pub fn try_to_remove(&mut self, inode_id: usize) -> bool {
        self.inodes.remove(&inode_id).is_some()
    } 
}
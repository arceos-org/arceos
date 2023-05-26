use super::config::BLOCK_SIZE;
use crate::block_cache_manager::BlockCacheManager;
use crate::mutex::SpinMutex;
use log::*;
/// A bitmap block
type BitmapBlock = [u64; BLOCK_SIZE / 8];
/// Number of bits in a block
const BLOCK_BITS: usize = BLOCK_SIZE * 8;
/// A bitmap
pub struct Bitmap {
    block_id: usize,
    offset: usize,
}

impl Bitmap {
    /// A new bitmap from start block id and number of blocks
    pub fn new(block_id: usize, offset: usize) -> Self {
        Self { block_id, offset }
    }
    /// Allocate a new block from a block device
    pub fn alloc(&self, manager: &SpinMutex<BlockCacheManager>) -> Option<usize> {
        let bitmap_block = manager.lock().get_block_cache(self.block_id);
        let bit = bitmap_block
            .lock()
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                if let Some((bits64_pos, inner_pos)) = bitmap_block
                    .iter()
                    .enumerate()
                    .find(|(_, bits64)| **bits64 != u64::MAX)
                    .map(|(bits64_pos, bits64)| (bits64_pos, bits64.trailing_ones() as usize))
                {
                    // modify cache
                    bitmap_block[bits64_pos] |= 1u64 << inner_pos;
                    Some(self.offset + bits64_pos * 64 + inner_pos as usize)
                } else {
                    None
                }
            });
        manager.lock().release_block(bitmap_block);
        bit
    }
    /// Test whether a bit is allocated
    pub fn test(&self, manager: &SpinMutex<BlockCacheManager>, bit: usize) -> bool {
        let mut res: bool = false;
        let (bits64_pos, inner_pos) = self.decomposition(bit);
        let bitmap_block = manager.lock().get_block_cache(self.block_id);
        bitmap_block.lock().read(0, |bitmap_block: &BitmapBlock| {
            res = bitmap_block[bits64_pos] & (1u64 << inner_pos) > 0;
        });
        manager.lock().release_block(bitmap_block);
        res
    }
    /// Deallocate a block
    pub fn dealloc(&self, manager: &SpinMutex<BlockCacheManager>, bit: usize) {
        let (bits64_pos, inner_pos) = self.decomposition(bit);
        let bitmap_block = manager.lock().get_block_cache(self.block_id);
        bitmap_block
            .lock()
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                assert!(bitmap_block[bits64_pos] & (1u64 << inner_pos) > 0);
                bitmap_block[bits64_pos] -= 1u64 << inner_pos;
            });
        manager.lock().release_block(bitmap_block);
    }
    /// Allocate a block no matter what it originally is
    #[allow(dead_code)]
    pub fn alloc_exact(&self, manager: &SpinMutex<BlockCacheManager>, bit: usize) {
        let (bits64_pos, inner_pos) = self.decomposition(bit);
        let bitmap_block = manager.lock().get_block_cache(self.block_id);
        bitmap_block
            .lock()
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                bitmap_block[bits64_pos] |= 1u64 << inner_pos;
            });
        manager.lock().release_block(bitmap_block);
    }

    /// Range allocation [start, end) (should only be used in creating file system)
    #[allow(dead_code)]
    pub fn range_alloc(
        &self,
        manager: &SpinMutex<BlockCacheManager>,
        mut start: usize,
        mut end: usize,
    ) {
        debug!("range_alloc {} {}", start, end);
        assert!(start < end);
        assert!(start >= self.minimum());
        assert!(end <= self.maximum());

        start -= self.minimum();
        end -= self.minimum();

        let bitmap_block = manager.lock().get_block_cache(self.block_id);
        bitmap_block
            .lock()
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                for (pos, inner) in bitmap_block.iter_mut().enumerate() {
                    for inner_pos in 0..64 as usize {
                        let idx = pos * 64 + inner_pos;
                        if idx >= start && idx < end {
                            *inner |= 1u64 << inner_pos;
                        }
                    }
                }
            });
        manager.lock().release_block(bitmap_block);
    }

    /// Get the max number of allocatable blocks
    pub fn maximum(&self) -> usize {
        self.offset + BLOCK_BITS
    }

    /// Get the min number of allocatable blocks
    pub fn minimum(&self) -> usize {
        self.offset
    }

    /// Decompose bits into (bits64_pos, inner_pos)
    fn decomposition(&self, mut bit: usize) -> (usize, usize) {
        assert!(bit >= self.minimum() && bit < self.maximum());
        bit -= self.minimum();
        (bit / 64, bit % 64)
    }
}

//! Buddy system page allocator implementation.  

use alloc::vec;
use alloc::vec::Vec;
use allocator::{AllocError, AllocResult, BaseAllocator, PageAllocator};
use core::cmp;
use memory_addr::{align_down, align_up, is_aligned};

const MAX_ORDER: usize = 20;
const MAX_ALIGN_1GB: usize = 0x4000_0000;

/// A page-granularity memory allocator based on the buddy system algorithm.  
///  
/// It maintains free lists for each order (power of 2 sizes) and supports  
/// efficient allocation and deallocation with automatic coalescing.  
///  
/// The `PAGE_SIZE` must be a power of two.  
pub struct BuddyPageAllocator<const PAGE_SIZE: usize> {
    /// Base address of the memory region  
    base: usize,
    /// Total number of pages  
    total_pages: usize,
    /// Number of pages currently in use  
    used_pages: usize,
    /// Free lists for each order (0 to MAX_ORDER)  
    free_lists: [Vec<usize>; MAX_ORDER + 1],
    /// Bitmap to track allocated status of each page  
    allocated: Vec<bool>,
}

impl<const PAGE_SIZE: usize> BuddyPageAllocator<PAGE_SIZE> {
    /// Creates a new empty `BuddyPageAllocator`.  
    pub const fn new() -> Self {
        const EMPTY_VEC: Vec<usize> = Vec::new();
        Self {
            base: 0,
            total_pages: 0,
            used_pages: 0,
            free_lists: [EMPTY_VEC; MAX_ORDER + 1],
            allocated: Vec::new(),
        }
    }

    /// Get the order (log2) of a size, rounded up  
    fn size_to_order(size: usize) -> usize {
        if size <= 1 {
            0
        } else {
            (size - 1).ilog2() as usize + 1
        }
    }

    /// Get the buddy address of a block at given order  
    fn get_buddy_addr(&self, addr: usize, order: usize) -> usize {
        let block_size = 1 << order;
        let page_idx = (addr - self.base) / PAGE_SIZE;
        let buddy_idx = page_idx ^ block_size;
        self.base + buddy_idx * PAGE_SIZE
    }

    /// Check if a block is completely free  
    fn is_block_free(&self, addr: usize, order: usize) -> bool {
        let block_size = 1 << order;
        let start_page = (addr - self.base) / PAGE_SIZE;

        for i in 0..block_size {
            if start_page + i >= self.allocated.len() || self.allocated[start_page + i] {
                return false;
            }
        }
        true
    }

    /// Mark a block as allocated or free  
    fn set_block_allocated(&mut self, addr: usize, order: usize, allocated: bool) {
        let block_size = 1 << order;
        let start_page = (addr - self.base) / PAGE_SIZE;

        for i in 0..block_size {
            if start_page + i < self.allocated.len() {
                self.allocated[start_page + i] = allocated;
            }
        }
    }

    /// Remove a block from the free list of given order  
    fn remove_from_free_list(&mut self, addr: usize, order: usize) -> bool {
        if let Some(pos) = self.free_lists[order].iter().position(|&x| x == addr) {
            self.free_lists[order].remove(pos);
            true
        } else {
            false
        }
    }

    /// Add a block to the free list of given order  
    fn add_to_free_list(&mut self, addr: usize, order: usize) {
        self.free_lists[order].push(addr);
    }

    /// Check if address is properly aligned for the given order  
    fn is_aligned_for_order(&self, addr: usize, order: usize) -> bool {
        let block_size = 1 << order;
        let page_idx = (addr - self.base) / PAGE_SIZE;
        page_idx % block_size == 0
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for BuddyPageAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        assert!(PAGE_SIZE.is_power_of_two());

        // Align the memory region to page boundaries
        let end = align_down(start + size, PAGE_SIZE);
        let start = align_up(start, PAGE_SIZE);
        self.total_pages = (end - start) / PAGE_SIZE;

        // Calculate base address aligned to 1GB for consistency with BitmapPageAllocator
        self.base = align_down(start, MAX_ALIGN_1GB);

        // Initialize tracking structures
        self.used_pages = 0;
        self.allocated = vec![false; self.total_pages];

        // Clear all free lists
        for list in &mut self.free_lists {
            list.clear();
        }

        // Add the entire memory region to appropriate free lists
        let mut remaining_pages = self.total_pages;
        let mut current_addr = start;

        while remaining_pages > 0 {
            // Find the largest order that fits
            let max_order_for_size = if remaining_pages.is_power_of_two() {
                remaining_pages.ilog2() as usize
            } else {
                (remaining_pages.next_power_of_two() >> 1).ilog2() as usize
            };

            // Also consider alignment constraints
            let page_idx = (current_addr - self.base) / PAGE_SIZE;
            let max_order_for_align = if page_idx == 0 {
                MAX_ORDER
            } else {
                page_idx.trailing_zeros() as usize
            };

            let order = cmp::min(cmp::min(max_order_for_size, max_order_for_align), MAX_ORDER);
            let block_size = 1 << order;

            if block_size <= remaining_pages {
                self.add_to_free_list(current_addr, order);
                current_addr += block_size * PAGE_SIZE;
                remaining_pages -= block_size;
            } else {
                // Fallback to smaller blocks
                let order = remaining_pages.ilog2() as usize;
                let block_size = 1 << order;
                self.add_to_free_list(current_addr, order);
                current_addr += block_size * PAGE_SIZE;
                remaining_pages -= block_size;
            }
        }
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        // Not supported for simplicity, similar to BitmapPageAllocator
        Err(AllocError::NoMemory)
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for BuddyPageAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if num_pages == 0 {
            return Err(AllocError::InvalidParam);
        }

        // Validate alignment similar to BitmapPageAllocator
        if align_pow2 > MAX_ALIGN_1GB || !is_aligned(align_pow2, PAGE_SIZE) {
            return Err(AllocError::InvalidParam);
        }

        let align_pages = align_pow2 / PAGE_SIZE;
        if !align_pages.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }

        // Calculate required order
        let required_order = Self::size_to_order(num_pages);
        let align_order = if align_pages > 1 {
            align_pages.ilog2() as usize
        } else {
            0
        };
        let order = cmp::max(required_order, align_order);

        // Find a free block of sufficient size
        for current_order in order..=MAX_ORDER {
            if !self.free_lists[current_order].is_empty() {
                // Find a properly aligned block
                let mut found_addr = None;
                for &addr in &self.free_lists[current_order] {
                    if is_aligned(addr, align_pow2) {
                        found_addr = Some(addr);
                        break;
                    }
                }

                if let Some(addr) = found_addr {
                    // Remove from free list
                    self.remove_from_free_list(addr, current_order);

                    // Split down to required order
                    let split_addr = addr;
                    for split_order in (order..current_order).rev() {
                        let buddy_addr = self.get_buddy_addr(split_addr, split_order);
                        self.add_to_free_list(buddy_addr, split_order);
                    }

                    // Mark as allocated
                    self.set_block_allocated(addr, order, true);
                    self.used_pages += 1 << order;
                    return Ok(addr);
                }
            }
        }

        Err(AllocError::NoMemory)
    }

    fn alloc_pages_at(
        &mut self,
        base: usize,
        num_pages: usize,
        align_pow2: usize,
    ) -> AllocResult<usize> {
        // Validate parameters
        if align_pow2 > MAX_ALIGN_1GB
            || !is_aligned(align_pow2, PAGE_SIZE)
            || !is_aligned(base, align_pow2)
            || num_pages == 0
        {
            return Err(AllocError::InvalidParam);
        }

        let order = Self::size_to_order(num_pages);

        // Check if the requested block is available and properly aligned
        if !self.is_aligned_for_order(base, order) || !self.is_block_free(base, order) {
            return Err(AllocError::NoMemory);
        }

        // This is a simplified implementation - in practice, you'd need to
        // handle splitting larger blocks that contain this address
        self.set_block_allocated(base, order, true);
        self.used_pages += 1 << order;
        Ok(base)
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        assert!(
            is_aligned(pos, Self::PAGE_SIZE),
            "pos must be aligned to PAGE_SIZE"
        );

        if num_pages == 0 {
            return;
        }

        let order = Self::size_to_order(num_pages);
        self.set_block_allocated(pos, order, false);
        self.used_pages -= 1 << order;

        // Try to coalesce with buddy blocks
        let mut current_addr = pos;
        let mut current_order = order;

        while current_order < MAX_ORDER {
            let buddy_addr = self.get_buddy_addr(current_addr, current_order);

            // Check if buddy is free and properly aligned
            if self.is_block_free(buddy_addr, current_order)
                && self.remove_from_free_list(buddy_addr, current_order)
            {
                // Merge with buddy - the merged block starts at the lower address
                current_addr = cmp::min(current_addr, buddy_addr);
                current_order += 1;
            } else {
                break;
            }
        }

        // Add the (possibly merged) block to the appropriate free list
        self.add_to_free_list(current_addr, current_order);
    }

    fn total_pages(&self) -> usize {
        self.total_pages
    }

    fn used_pages(&self) -> usize {
        self.used_pages
    }

    fn available_pages(&self) -> usize {
        self.total_pages - self.used_pages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE_SIZE: usize = 4096;

    #[test]
    fn test_buddy_page_allocator_basic() {
        let mut allocator = BuddyPageAllocator::<PAGE_SIZE>::new();
        allocator.init(PAGE_SIZE, PAGE_SIZE);

        assert_eq!(allocator.total_pages(), 1);
        assert_eq!(allocator.used_pages(), 0);
        assert_eq!(allocator.available_pages(), 1);

        let addr = allocator.alloc_pages(1, PAGE_SIZE).unwrap();
        assert_eq!(addr, PAGE_SIZE);
        assert_eq!(allocator.used_pages(), 1);
        assert_eq!(allocator.available_pages(), 0);

        allocator.dealloc_pages(addr, 1);
        assert_eq!(allocator.used_pages(), 0);
        assert_eq!(allocator.available_pages(), 1);
    }

    #[test]
    fn test_buddy_page_allocator_multiple_pages() {
        const SIZE_1M: usize = 1024 * 1024;
        let mut allocator = BuddyPageAllocator::<PAGE_SIZE>::new();
        allocator.init(0, SIZE_1M);

        let total_pages = SIZE_1M / PAGE_SIZE;
        assert_eq!(allocator.total_pages(), total_pages);

        // Test power-of-2 allocations
        let addr1 = allocator.alloc_pages(1, PAGE_SIZE).unwrap();
        assert_eq!(allocator.used_pages(), 1);

        let addr2 = allocator.alloc_pages(2, PAGE_SIZE).unwrap();
        assert_eq!(allocator.used_pages(), 3);

        let addr4 = allocator.alloc_pages(4, PAGE_SIZE).unwrap();
        assert_eq!(allocator.used_pages(), 7);

        // Test deallocation and coalescing
        allocator.dealloc_pages(addr1, 1);
        assert_eq!(allocator.used_pages(), 6);

        allocator.dealloc_pages(addr2, 2);
        assert_eq!(allocator.used_pages(), 4);

        allocator.dealloc_pages(addr4, 4);
        assert_eq!(allocator.used_pages(), 0);
    }

    #[test]
    fn test_buddy_page_allocator_alignment() {
        const SIZE_1M: usize = 1024 * 1024;
        let mut allocator = BuddyPageAllocator::<PAGE_SIZE>::new();
        allocator.init(0, SIZE_1M);

        // Test various alignments
        let alignments = [PAGE_SIZE, PAGE_SIZE * 2, PAGE_SIZE * 4, PAGE_SIZE * 8];

        for &align in &alignments {
            let addr = allocator.alloc_pages(1, align).unwrap();
            assert!(is_aligned(addr, align));
            allocator.dealloc_pages(addr, 1);
        }
    }

    #[test]
    fn test_buddy_page_allocator_coalescing() {
        const SIZE_1M: usize = 1024 * 1024;
        let mut allocator = BuddyPageAllocator::<PAGE_SIZE>::new();
        allocator.init(0, SIZE_1M);

        // Allocate 4 single pages
        let addr1 = allocator.alloc_pages(1, PAGE_SIZE).unwrap();
        let addr2 = allocator.alloc_pages(1, PAGE_SIZE).unwrap();
        let addr3 = allocator.alloc_pages(1, PAGE_SIZE).unwrap();
        let addr4 = allocator.alloc_pages(1, PAGE_SIZE).unwrap();

        assert_eq!(allocator.used_pages(), 4);

        // Free them in reverse order to test coalescing
        allocator.dealloc_pages(addr4, 1);
        allocator.dealloc_pages(addr3, 1);
        allocator.dealloc_pages(addr2, 1);
        allocator.dealloc_pages(addr1, 1);

        assert_eq!(allocator.used_pages(), 0);
        assert_eq!(allocator.available_pages(), SIZE_1M / PAGE_SIZE);

        // Should be able to allocate a large block now
        let large_addr = allocator.alloc_pages(16, PAGE_SIZE).unwrap();
        assert_eq!(allocator.used_pages(), 16);
        allocator.dealloc_pages(large_addr, 16);
    }

    #[test]
    fn test_buddy_page_allocator_size_2g() {
        const SIZE_1G: usize = 1024 * 1024 * 1024;
        const SIZE_2G: usize = 2 * SIZE_1G;
        const TEST_BASE_ADDR: usize = SIZE_1G + PAGE_SIZE;

        let mut allocator = BuddyPageAllocator::<PAGE_SIZE>::new();
        allocator.init(TEST_BASE_ADDR, SIZE_2G);

        let mut num_pages = 1;
        // Test allocation and deallocation of 1, 2, 4, 8 pages
        while num_pages <= 8 {
            assert_eq!(allocator.total_pages(), SIZE_2G / PAGE_SIZE);
            let used_before = allocator.used_pages();

            let addr = allocator.alloc_pages(num_pages, PAGE_SIZE).unwrap();
            assert!(is_aligned(addr, PAGE_SIZE));
            assert_eq!(allocator.used_pages(), used_before + num_pages);

            allocator.dealloc_pages(addr, num_pages);
            assert_eq!(allocator.used_pages(), used_before);

            num_pages *= 2;
        }
    }

    #[test]
    fn test_buddy_page_allocator_fragmentation() {
        const SIZE_64K: usize = 64 * 1024;
        let mut allocator = BuddyPageAllocator::<PAGE_SIZE>::new();
        allocator.init(0, SIZE_64K);

        let total_pages = SIZE_64K / PAGE_SIZE;
        let mut addrs = Vec::new();

        // Allocate all pages as single pages
        for _ in 0..total_pages {
            if let Ok(addr) = allocator.alloc_pages(1, PAGE_SIZE) {
                addrs.push(addr);
            }
        }

        assert_eq!(allocator.used_pages(), total_pages);
        assert_eq!(allocator.available_pages(), 0);

        // Free every other page to create fragmentation
        for (i, &addr) in addrs.iter().enumerate() {
            if i % 2 == 0 {
                allocator.dealloc_pages(addr, 1);
            }
        }

        assert_eq!(allocator.used_pages(), total_pages / 2);

        // Try to allocate a 2-page block - should fail due to fragmentation
        assert!(allocator.alloc_pages(2, PAGE_SIZE).is_err());

        // Free remaining pages
        for (i, &addr) in addrs.iter().enumerate() {
            if i % 2 == 1 {
                allocator.dealloc_pages(addr, 1);
            }
        }

        assert_eq!(allocator.used_pages(), 0);
    }

    #[test]
    fn test_buddy_page_allocator_invalid_params() {
        let mut allocator = BuddyPageAllocator::<PAGE_SIZE>::new();
        allocator.init(PAGE_SIZE, PAGE_SIZE * 16);

        // Test invalid number of pages
        assert!(allocator.alloc_pages(0, PAGE_SIZE).is_err());

        // Test invalid alignment
        assert!(allocator.alloc_pages(1, PAGE_SIZE - 1).is_err());
        assert!(allocator.alloc_pages(1, MAX_ALIGN_1GB + 1).is_err());

        // Test non-power-of-2 alignment
        assert!(allocator.alloc_pages(1, PAGE_SIZE * 3).is_err());
    }
}

/// Benchmark tests comparing BitmapPageAllocator and BuddyPageAllocator      
#[cfg(test)]
mod benchmark_tests {
    #[cfg(feature = "buddy-page")]
    use crate::buddy_page::BuddyPageAllocator;
    use alloc::vec::Vec;
    use allocator::{BaseAllocator, BitmapPageAllocator, PageAllocator};
    use core::time::Duration;

    const LARGE_SIZE: usize = 2 * 1024 * 1024; // 6MB (1536 pages)  
    const PAGE_SIZE: usize = 4096;

    /// Test fragmentation with different allocation patterns  
    #[test]
    #[cfg(feature = "buddy-page")]
    fn test_fragmentation_comparison() {
        println!("=== Fragmentation Comparison ===");

        // Test 1: Random small allocations
        let bitmap_frag1 =
            test_fragmentation_pattern(&mut create_bitmap_allocator(), "random_small");
        let buddy_frag1 = test_fragmentation_pattern(&mut create_buddy_allocator(), "random_small");

        // Test 2: Mixed size allocations
        let bitmap_frag2 = test_fragmentation_pattern(&mut create_bitmap_allocator(), "mixed_size");
        let buddy_frag2 = test_fragmentation_pattern(&mut create_buddy_allocator(), "mixed_size");

        // Test 3: Power-of-2 allocations (favors buddy)
        let bitmap_frag3 = test_fragmentation_pattern(&mut create_bitmap_allocator(), "power_of_2");
        let buddy_frag3 = test_fragmentation_pattern(&mut create_buddy_allocator(), "power_of_2");

        println!("Random Small Pattern:");
        println!("  Bitmap: {:.1}%, Buddy: {:.1}%", bitmap_frag1, buddy_frag1);
        println!("Mixed Size Pattern:");
        println!("  Bitmap: {:.1}%, Buddy: {:.1}%", bitmap_frag2, buddy_frag2);
        println!("Power-of-2 Pattern:");
        println!("  Bitmap: {:.1}%, Buddy: {:.1}%", bitmap_frag3, buddy_frag3);
    }

    /// Test coalescing efficiency  
    #[test]
    #[cfg(feature = "buddy-page")]
    fn test_coalescing_efficiency() {
        println!("=== Coalescing Efficiency Test ===");

        let bitmap_efficiency = test_coalescing(&mut create_bitmap_allocator());
        let buddy_efficiency = test_coalescing(&mut create_buddy_allocator());

        println!("Bitmap coalescing efficiency: {:.1}%", bitmap_efficiency);
        println!("Buddy coalescing efficiency: {:.1}%", buddy_efficiency);

        if (bitmap_efficiency - buddy_efficiency).abs() > 5.0 {
            let better = if bitmap_efficiency > buddy_efficiency {
                "Bitmap"
            } else {
                "Buddy"
            };
            println!(
                "{} shows better coalescing by {:.1}% points",
                better,
                (bitmap_efficiency - buddy_efficiency).abs()
            );
        }
    }

    fn create_bitmap_allocator() -> BitmapPageAllocator<PAGE_SIZE> {
        let mut allocator = BitmapPageAllocator::<PAGE_SIZE>::new();
        allocator.init(0, LARGE_SIZE);
        allocator
    }

    #[cfg(feature = "buddy-page")]
    fn create_buddy_allocator() -> BuddyPageAllocator<PAGE_SIZE> {
        let mut allocator = BuddyPageAllocator::<PAGE_SIZE>::new();
        allocator.init(0, LARGE_SIZE);
        allocator
    }

    fn test_fragmentation_pattern<T: PageAllocator>(allocator: &mut T, pattern: &str) -> f64 {
        let mut allocated_addrs = Vec::new();

        // Different allocation patterns
        match pattern {
            "random_small" => {
                // Allocate many small blocks with pseudo-random sizes
                for i in 0..80 {
                    let size = 1 + (i * 7 + 13) % 8; // Pseudo-random 1-8 pages  
                    if let Ok(addr) = allocator.alloc_pages(size, PAGE_SIZE) {
                        allocated_addrs.push((addr, size));
                    }
                }
            }
            "mixed_size" => {
                // Mix of small, medium, and large allocations
                for i in 0..150 {
                    let size = match i % 7 {
                        0..=3 => 1 + i % 4,  // Small: 1-4 pages
                        4..=5 => 8 + i % 16, // Medium: 8-23 pages
                        _ => 32 + i % 32,    // Large: 32-63 pages
                    };
                    if let Ok(addr) = allocator.alloc_pages(size, PAGE_SIZE) {
                        allocated_addrs.push((addr, size));
                    }
                }
            }
            "power_of_2" => {
                // Power-of-2 sizes (should favor buddy allocator)
                for i in 0..120 {
                    let size = 1 << (i % 7); // 1, 2, 4, 8, 16, 32, 64 pages  
                    if let Ok(addr) = allocator.alloc_pages(size, PAGE_SIZE) {
                        allocated_addrs.push((addr, size));
                    }
                }
            }
            _ => {}
        }

        // Free blocks in a pattern that creates fragmentation
        for (i, &(addr, size)) in allocated_addrs.iter().enumerate() {
            // Free every 3rd and 5th block to create irregular gaps
            if i % 3 == 0 || i % 5 == 0 {
                allocator.dealloc_pages(addr, size);
            }
        }

        // Test large block allocation success rate
        let mut successful_allocs = 0;
        let test_size = 24; // 24 pages (96KB)  
        let attempts = 12;

        for _ in 0..attempts {
            if allocator.alloc_pages(test_size, PAGE_SIZE).is_ok() {
                successful_allocs += 1;
            }
        }

        (attempts - successful_allocs) as f64 / attempts as f64 * 100.0
    }

    fn test_coalescing<T: PageAllocator>(allocator: &mut T) -> f64 {
        let mut allocated_blocks = Vec::new();

        // Allocate 100 single pages
        for _ in 0..100 {
            if let Ok(addr) = allocator.alloc_pages(1, PAGE_SIZE) {
                allocated_blocks.push(addr);
            }
        }

        // Free all blocks
        for addr in allocated_blocks {
            allocator.dealloc_pages(addr, 1);
        }

        // Test coalescing by trying to allocate one large block
        let max_possible = 100; // We freed exactly 100 pages  
        if allocator.alloc_pages(max_possible, PAGE_SIZE).is_ok() {
            100.0 // Perfect coalescing  
        } else {
            // Try smaller sizes to see what's actually available
            let mut max_successful_size = 0;
            for size in [1, 2, 4, 8, 16, 32, 64, 80, 90, 95, 99] {
                if allocator.alloc_pages(size, PAGE_SIZE).is_ok() {
                    max_successful_size = size;
                } else {
                    break;
                }
            }
            (max_successful_size as f64 / max_possible as f64) * 100.0
        }
    }

    // Simple time measurement (microseconds)
    fn get_time_us() -> u64 {
        // This is a placeholder - in real implementation you'd use proper timing
        // For testing purposes, we'll use a simple counter
        static mut COUNTER: u64 = 0;
        unsafe {
            COUNTER += 1;
            COUNTER
        }
    }
}

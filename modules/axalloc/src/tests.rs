use super::*;

#[repr(align(4096))]
struct TestHeap([u8; 64 * 1024]);

#[test]
fn global_alloc_returns_null_on_oom_and_remains_usable() {
    let mut heap = TestHeap([0; 64 * 1024]);
    // Large page allocator configurations have bitmaps exceeding a test
    // thread's stack. Keep this test's independent allocator in static storage.
    static ALLOCATOR: GlobalAllocator = GlobalAllocator::new();
    let allocator = &ALLOCATOR;
    // Initialize only the byte heap. An empty page allocator makes expansion
    // fail deterministically, without depending on host virtual addresses.
    allocator
        .balloc
        .lock()
        .init(heap.0.as_mut_ptr() as usize, heap.0.len());

    let large = Layout::from_size_align(1024 * 1024, 8).unwrap();
    assert!(allocator.alloc(large).is_err());
    assert!(unsafe { GlobalAlloc::alloc(allocator, large) }.is_null());

    let small = Layout::from_size_align(64, 8).unwrap();
    let ptr = unsafe { GlobalAlloc::alloc(allocator, small) };
    assert!(!ptr.is_null());
    let used = allocator.used_bytes();
    unsafe {
        ptr.write_bytes(0xa5, small.size());
        for i in 0..small.size() {
            assert_eq!(*ptr.add(i), 0xa5);
        }
        GlobalAlloc::dealloc(allocator, ptr, small);
    }
    // Slab may retain a backing page after the user allocation is freed.
    assert_eq!(allocator.used_bytes(), used - small.size());
}

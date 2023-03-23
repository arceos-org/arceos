use super::*;
use alloc::alloc::Layout;
use core::mem::{align_of, size_of};

const HEAP_SIZE: usize = 16 * 4096;
const BIG_HEAP_SIZE: usize = HEAP_SIZE * 10;

#[repr(align(4096))]
struct TestHeap {
    heap_space: [u8; HEAP_SIZE],
}

#[repr(align(4096))]
struct TestBigHeap {
    heap_space: [u8; BIG_HEAP_SIZE],
}

fn new_heap() -> Heap {
    let test_heap = TestHeap {
        heap_space: [0u8; HEAP_SIZE],
    };
    let heap = unsafe { Heap::new(&test_heap.heap_space[0] as *const u8 as usize, HEAP_SIZE) };
    heap
}

fn new_big_heap() -> Heap {
    let test_heap = TestBigHeap {
        heap_space: [0u8; BIG_HEAP_SIZE],
    };
    let heap = unsafe {
        Heap::new(
            &test_heap.heap_space[0] as *const u8 as usize,
            BIG_HEAP_SIZE,
        )
    };
    heap
}

#[test]
fn oom() {
    let mut heap = new_heap();
    let layout = Layout::from_size_align(HEAP_SIZE + 1, align_of::<usize>());
    let addr = heap.allocate(layout.unwrap());
    assert!(addr.is_err());
}

#[test]
fn allocate_double_usize() {
    let mut heap = new_heap();
    let size = size_of::<usize>() * 2;
    let layout = Layout::from_size_align(size, align_of::<usize>());
    let addr = heap.allocate(layout.unwrap());
    assert!(addr.is_ok());
}

#[test]
fn allocate_and_free_double_usize() {
    let mut heap = new_heap();
    let layout = Layout::from_size_align(size_of::<usize>() * 2, align_of::<usize>()).unwrap();
    let addr = heap.allocate(layout.clone());
    assert!(addr.is_ok());
    let addr = addr.unwrap();
    unsafe {
        *(addr as *mut (usize, usize)) = (0xdeafdeadbeafbabe, 0xdeafdeadbeafbabe);

        heap.deallocate(addr, layout.clone());
    }
}

#[test]
fn reallocate_double_usize() {
    let mut heap = new_heap();

    let layout = Layout::from_size_align(size_of::<usize>() * 2, align_of::<usize>()).unwrap();

    let x = heap.allocate(layout.clone()).unwrap();
    unsafe {
        heap.deallocate(x, layout.clone());
    }

    let y = heap.allocate(layout.clone()).unwrap();
    unsafe {
        heap.deallocate(y, layout.clone());
    }

    assert_eq!(x as usize, y as usize);
}

#[test]
fn allocate_multiple_sizes() {
    let mut heap = new_heap();
    let base_size = size_of::<usize>();
    let base_align = align_of::<usize>();

    let layout_1 = Layout::from_size_align(base_size * 2, base_align).unwrap();
    let layout_2 = Layout::from_size_align(base_size * 3, base_align).unwrap();
    let layout_3 = Layout::from_size_align(base_size * 3, base_align * 8).unwrap();
    let layout_4 = Layout::from_size_align(base_size * 10, base_align).unwrap();

    let x = heap.allocate(layout_1.clone()).unwrap();
    let y = heap.allocate(layout_2.clone()).unwrap();
    let z = heap.allocate(layout_3.clone()).unwrap();

    unsafe {
        heap.deallocate(x, layout_1.clone());
    }

    let a = heap.allocate(layout_4.clone()).unwrap();
    let b = heap.allocate(layout_1.clone()).unwrap();

    unsafe {
        heap.deallocate(y, layout_2);
        heap.deallocate(z, layout_3);
        heap.deallocate(a, layout_4);
        heap.deallocate(b, layout_1);
    }
}

#[test]
fn allocate_one_4096_block() {
    let mut heap = new_big_heap();
    let base_size = size_of::<usize>();
    let base_align = align_of::<usize>();

    let layout = Layout::from_size_align(base_size * 512, base_align).unwrap();

    let x = heap.allocate(layout.clone()).unwrap();

    unsafe {
        heap.deallocate(x, layout.clone());
    }
}

#[test]
fn allocate_multiple_4096_blocks() {
    let mut heap = new_big_heap();
    let base_size = size_of::<usize>();
    let base_align = align_of::<usize>();

    let layout = Layout::from_size_align(base_size * 512, base_align).unwrap();
    let layout_2 = Layout::from_size_align(base_size * 1024, base_align).unwrap();

    let _x = heap.allocate(layout.clone()).unwrap();
    let y = heap.allocate(layout.clone()).unwrap();
    let z = heap.allocate(layout.clone()).unwrap();

    unsafe {
        heap.deallocate(y, layout.clone());
    }

    let a = heap.allocate(layout.clone()).unwrap();
    let _b = heap.allocate(layout.clone()).unwrap();

    unsafe {
        heap.deallocate(a, layout.clone());
        heap.deallocate(z, layout.clone());
    }
    let c = heap.allocate(layout_2.clone()).unwrap();
    let _d = heap.allocate(layout.clone()).unwrap();
    unsafe {
        *(c as *mut (usize, usize)) = (0xdeafdeadbeafbabe, 0xdeafdeadbeafbabe);
    }
}

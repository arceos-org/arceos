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

    unsafe { Heap::new(&test_heap.heap_space[0] as *const u8 as usize, HEAP_SIZE) }
}

fn new_big_heap() -> Heap {
    let test_heap = TestBigHeap {
        heap_space: [0u8; BIG_HEAP_SIZE],
    };

    unsafe {
        Heap::new(
            &test_heap.heap_space[0] as *const u8 as usize,
            BIG_HEAP_SIZE,
        )
    }
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
    let addr = heap.allocate(layout);
    assert!(addr.is_ok());
    let addr = addr.unwrap();
    unsafe {
        *(addr as *mut (usize, usize)) = (0xdeafdeadbeafbabe, 0xdeafdeadbeafbabe);

        heap.deallocate(addr, layout);
    }
}

#[test]
fn reallocate_double_usize() {
    let mut heap = new_heap();

    let layout = Layout::from_size_align(size_of::<usize>() * 2, align_of::<usize>()).unwrap();

    let x = heap.allocate(layout).unwrap();
    unsafe {
        heap.deallocate(x, layout);
    }

    let y = heap.allocate(layout).unwrap();
    unsafe {
        heap.deallocate(y, layout);
    }

    assert_eq!({ x }, { y });
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

    let x = heap.allocate(layout_1).unwrap();
    let y = heap.allocate(layout_2).unwrap();
    let z = heap.allocate(layout_3).unwrap();

    unsafe {
        heap.deallocate(x, layout_1);
    }

    let a = heap.allocate(layout_4).unwrap();
    let b = heap.allocate(layout_1).unwrap();

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

    let x = heap.allocate(layout).unwrap();

    unsafe {
        heap.deallocate(x, layout);
    }
}

#[test]
fn allocate_multiple_4096_blocks() {
    let mut heap = new_big_heap();
    let base_size = size_of::<usize>();
    let base_align = align_of::<usize>();

    let layout = Layout::from_size_align(base_size * 512, base_align).unwrap();
    let layout_2 = Layout::from_size_align(base_size * 1024, base_align).unwrap();

    let _x = heap.allocate(layout).unwrap();
    let y = heap.allocate(layout).unwrap();
    let z = heap.allocate(layout).unwrap();

    unsafe {
        heap.deallocate(y, layout);
    }

    let a = heap.allocate(layout).unwrap();
    let _b = heap.allocate(layout).unwrap();

    unsafe {
        heap.deallocate(a, layout);
        heap.deallocate(z, layout);
    }
    let c = heap.allocate(layout_2).unwrap();
    let _d = heap.allocate(layout).unwrap();
    unsafe {
        *(c as *mut (usize, usize)) = (0xdeafdeadbeafbabe, 0xdeafdeadbeafbabe);
    }
}

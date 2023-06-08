extern crate alloc;
use super::{rand_u32, rand_usize};
use alloc::alloc::{alloc, dealloc};
use alloc::vec::Vec;
use core::alloc::Layout;

/// new aligned memory
pub fn new_mem(size: usize, align: usize) -> usize {
    unsafe {
        let ptr = alloc(Layout::from_size_align_unchecked(size, align));
        ptr as usize
    }
}

/// align test
pub fn align_test() {
    let mut v = Vec::new();
    let mut v2 = Vec::new();
    let mut v3 = Vec::new();
    let mut p = Vec::new();
    let n = 50000;
    let mut cnt = 0;
    let mut nw = 0;
    for _ in 0..n {
        if (rand_u32() % 3 != 0) | (nw == 0) {
            //插入一个块
            let size = (((1 << (rand_u32() & 15)) as f64)
                * (1.0 + (rand_u32() as f64) / (0xffffffff_u32 as f64)))
                as usize;
            let align = (1 << (rand_u32() & 7)) as usize;
            let addr = new_mem(size, align);
            v.push(addr);
            assert!((addr & (align - 1)) == 0, "align not correct.");
            v2.push(size);
            v3.push(align);
            p.push(cnt);
            cnt += 1;
            nw += 1;
        } else {
            //删除一个块
            let idx = rand_usize() % nw;
            let addr = v[p[idx]];
            let size = v2[p[idx]];
            let align = v3[p[idx]];
            unsafe {
                dealloc(
                    addr as *mut u8,
                    Layout::from_size_align_unchecked(size, align),
                );
            }
            nw -= 1;
            p[idx] = p[nw];
            p.pop();
        }
    }
    for idx in 0..nw {
        let addr = v[p[idx]];
        let size = v2[p[idx]];
        let align = v3[p[idx]];
        unsafe {
            dealloc(
                addr as *mut u8,
                Layout::from_size_align_unchecked(size, align),
            );
        }
    }
}

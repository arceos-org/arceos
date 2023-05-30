//! Allocator test for user mode test of crate `allocator`.

#![feature(ptr_alignment_type)]
#![no_std]
extern crate alloc;
use alloc::alloc::{alloc, dealloc};
mod basic_test;
use alloc::vec::Vec;
pub use basic_test::{basic_test, rand_u32, rand_usize, srand};
use core::{alloc::Layout, ffi::c_ulonglong};

#[link(name = "allocator_test")]
extern "C" {
    /// the start function of the mi_test
    pub fn mi_test_start(cb1: CallBackMalloc, cb2: CallBackMallocAligned, cb3: CallBackFree);
    /// the start function of the malloc_large_test
    pub fn malloc_large_test_start(
        cb1: CallBackMalloc,
        cb2: CallBackMallocAligned,
        cb3: CallBackFree,
    );
    /// the start function of the glibc_bench_test
    pub fn glibc_bench_test_start(
        cb1: CallBackMalloc,
        cb2: CallBackMallocAligned,
        cb3: CallBackFree,
    );
    /// the start function of the multi_thread_c_test
    pub fn multi_thread_c_test_start(
        cb1: CallBackMalloc,
        cb2: CallBackMallocAligned,
        cb3: CallBackFree,
    );
}

/// the call back function of malloc
pub type CallBackMalloc = unsafe extern "C" fn(size: c_ulonglong) -> c_ulonglong;
/// the call back function of malloc_aligned
pub type CallBackMallocAligned =
    unsafe extern "C" fn(size: c_ulonglong, align: c_ulonglong) -> c_ulonglong;
/// the call back function of free
pub type CallBackFree = unsafe extern "C" fn(ptr: c_ulonglong, size: c_ulonglong);

/// # Safety
pub unsafe extern "C" fn cb_malloc_func(size: c_ulonglong) -> c_ulonglong {
    let ptr = alloc(Layout::from_size_align_unchecked(size as usize, 8));
    ptr as c_ulonglong
}
/// # Safety
pub unsafe extern "C" fn cb_malloc_aligned_func(
    size: c_ulonglong,
    align: c_ulonglong,
) -> c_ulonglong {
    let ptr = alloc(Layout::from_size_align_unchecked(
        size as usize,
        align as usize,
    ));
    ptr as c_ulonglong
}
/// # Safety
pub unsafe extern "C" fn cb_free_func(ptr: c_ulonglong, size: c_ulonglong) {
    dealloc(
        ptr as *mut u8,
        Layout::from_size_align_unchecked(size as usize, 8),
    );
}

/// interface of mi test
pub fn mi_test() {
    unsafe {
        mi_test_start(cb_malloc_func, cb_malloc_aligned_func, cb_free_func);
    }
}
/// interface of malloc large test
pub fn malloc_large_test() {
    unsafe {
        malloc_large_test_start(cb_malloc_func, cb_malloc_aligned_func, cb_free_func);
    }
}
/// interface of glibc bench test
pub fn glibc_bench_test() {
    unsafe {
        glibc_bench_test_start(cb_malloc_func, cb_malloc_aligned_func, cb_free_func);
    }
}
/// interface of thread c test
pub fn multi_thread_c_test() {
    unsafe {
        multi_thread_c_test_start(cb_malloc_func, cb_malloc_aligned_func, cb_free_func);
    }
}

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

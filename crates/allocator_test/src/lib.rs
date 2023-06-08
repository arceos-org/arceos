//! Allocator test for user mode test of crate `allocator`.

#![feature(ptr_alignment_type)]
#![no_std]
extern crate alloc;
use alloc::alloc::{alloc, dealloc};
mod align_test;
mod basic_test;
pub use align_test::align_test;
pub use basic_test::basic_test;
use core::sync::atomic::{AtomicU64, Ordering::SeqCst};
use core::{alloc::Layout, ffi::c_ulonglong};

static SEED: AtomicU64 = AtomicU64::new(0xa2ce_a2ce);

/// Sets the seed for the random number generator.
pub fn srand(seed: u32) {
    SEED.store(seed.wrapping_sub(1) as u64, SeqCst);
}

/// Returns a 32-bit unsigned pseudo random interger.
pub fn rand_u32() -> u32 {
    let new_seed = SEED.load(SeqCst).wrapping_mul(6364136223846793005) + 1;
    SEED.store(new_seed, SeqCst);
    (new_seed >> 33) as u32
}

/// Return a usize pseudo random interger.
pub fn rand_usize() -> usize {
    ((rand_u32() as usize) << 32) | (rand_u32() as usize)
}

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

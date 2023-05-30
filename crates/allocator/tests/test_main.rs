#![feature(ptr_alignment_type)]
mod basic_test;
use basic_test::{basic_test, rand_u32, rand_usize, srand};
mod global_allocator;
use core::panic;
use core::sync::atomic::{AtomicUsize, Ordering};
use global_allocator::GLOBAL_ALLOCATOR;
use std::ffi::c_int;
use std::thread;
use std::time::Duration;
use std::vec::Vec;
use std::{alloc::Layout, ffi::c_ulonglong};
//use allocator_test::*;

#[link(name = "allocator_test")]
extern "C" {
    pub fn hello(a: c_int, cb: CallBack) -> c_int;
    pub fn mi_test_start(cb1: CallBackMalloc, cb2: CallBackMallocAligned, cb3: CallBackFree);
    pub fn malloc_large_test_start(
        cb1: CallBackMalloc,
        cb2: CallBackMallocAligned,
        cb3: CallBackFree,
    );
    pub fn glibc_bench_test_start(
        cb1: CallBackMalloc,
        cb2: CallBackMallocAligned,
        cb3: CallBackFree,
    );
    pub fn multi_thread_c_test_start(
        cb1: CallBackMalloc,
        cb2: CallBackMallocAligned,
        cb3: CallBackFree,
    );
}

pub type CallBack = unsafe extern "C" fn(c_int) -> c_int;
pub unsafe extern "C" fn cb_func(x: c_int) -> c_int {
    println!("hello rust! {:#?}", x);
    return x * x + 1;
}
pub fn call_back_test(x: c_int) {
    unsafe {
        let y = hello(x, cb_func);
        println!("rust call_back test passed! {:#?}", y);
    }
}

pub type CallBackMalloc = unsafe extern "C" fn(size: c_ulonglong) -> c_ulonglong;
pub type CallBackMallocAligned =
    unsafe extern "C" fn(size: c_ulonglong, align: c_ulonglong) -> c_ulonglong;
pub type CallBackFree = unsafe extern "C" fn(ptr: c_ulonglong, size: c_ulonglong);

pub unsafe extern "C" fn cb_malloc_func(size: c_ulonglong) -> c_ulonglong {
    if let Ok(ptr) = GLOBAL_ALLOCATOR.alloc(Layout::from_size_align_unchecked(size as usize, 8)) {
        return ptr as c_ulonglong;
    }
    panic!("alloc err.");
}
pub unsafe extern "C" fn cb_malloc_aligned_func(
    size: c_ulonglong,
    align: c_ulonglong,
) -> c_ulonglong {
    if let Ok(ptr) = GLOBAL_ALLOCATOR.alloc(Layout::from_size_align_unchecked(
        size as usize,
        align as usize,
    )) {
        return ptr as c_ulonglong;
    }
    panic!("alloc err.");
}
pub unsafe extern "C" fn cb_free_func(ptr: c_ulonglong, size: c_ulonglong) {
    GLOBAL_ALLOCATOR.dealloc(
        ptr as usize,
        Layout::from_size_align_unchecked(size as usize, 8),
    );
}

pub fn mi_test() {
    println!("Mi alloc test begin...");
    let t0 = std::time::Instant::now();
    unsafe {
        mi_test_start(cb_malloc_func, cb_malloc_aligned_func, cb_free_func);
    }
    let t1 = std::time::Instant::now();
    println!("time: {:#?}", t1 - t0);
    println!("Mi alloc test OK!");
}
pub fn malloc_large_test() {
    println!("Malloc large test begin...");
    let t0 = std::time::Instant::now();
    unsafe {
        malloc_large_test_start(cb_malloc_func, cb_malloc_aligned_func, cb_free_func);
    }
    let t1 = std::time::Instant::now();
    println!("time: {:#?}", t1 - t0);
    println!("Malloc large test OK!");
}
pub fn glibc_bench_test() {
    println!("Glibc bench test begin...");
    let t0 = std::time::Instant::now();
    unsafe {
        glibc_bench_test_start(cb_malloc_func, cb_malloc_aligned_func, cb_free_func);
    }
    let t1 = std::time::Instant::now();
    println!("time: {:#?}", t1 - t0);
    println!("Glibc bench test OK!");
}
pub fn multi_thread_c_test() {
    println!("Multi thread C test begin...");
    let t0 = std::time::Instant::now();
    unsafe {
        multi_thread_c_test_start(cb_malloc_func, cb_malloc_aligned_func, cb_free_func);
    }
    let t1 = std::time::Instant::now();
    println!("time: {:#?}", t1 - t0);
    println!("Multi thread C test OK!");
}

///memory chk
pub fn memory_chk() {
    unsafe {
        let tot = GLOBAL_ALLOCATOR.total_bytes() as f64;
        let used = GLOBAL_ALLOCATOR.used_bytes() as f64;
        let avail = GLOBAL_ALLOCATOR.available_bytes() as f64;
        println!("total memory: {:#?} MB", tot / 1048576.0);
        println!("used memory: {:#?} MB", used / 1048576.0);
        println!("available memory: {:#?} MB", avail / 1048576.0);
        println!("occupied memory: {:#?} MB", (tot - avail) / 1048576.0);
        println!(
            "extra memory rate: {:#?}%",
            (tot - avail - used) / (tot - avail) * 100.0
        );
    }
}

/// new aligned memory
pub fn new_mem(size: usize, align: usize) -> usize {
    unsafe {
        if let Ok(ptr) = GLOBAL_ALLOCATOR.alloc(Layout::from_size_align_unchecked(size, align)) {
            return ptr;
        }
        panic!("alloc err.");
    }
}

/// align test
pub fn align_test() {
    println!("Align alloc test begin...");
    let t0 = std::time::Instant::now();
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
                * (1.0 + (rand_u32() as f64) / (0xffffffff as u32 as f64)))
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
                GLOBAL_ALLOCATOR.dealloc(
                    addr,
                    Layout::from_size_align_unchecked(size as usize, align),
                );
            }
            nw -= 1;
            p[idx] = p[nw];
            p.pop();
        }
    }
    memory_chk();
    for idx in 0..nw {
        let addr = v[p[idx]];
        let size = v2[p[idx]];
        let align = v3[p[idx]];
        unsafe {
            GLOBAL_ALLOCATOR.dealloc(
                addr,
                Layout::from_size_align_unchecked(size as usize, align),
            );
        }
    }
    let t1 = std::time::Instant::now();
    println!("time: {:#?}", t1 - t0);
    println!("Align alloc test OK!");
}

const NUM_TASKS: usize = 10;
const MUN_TURN: usize = 100;
const NUM_ARRAY_PRE_THREAD: usize = 1000;

static mut MEMORY_POOL: Vec<AtomicUsize> = Vec::new(); //NUM_TASKS * NUM_ARRAY_PRE_THREAD] = [AtomicUsize::new(0); NUM_TASKS * NUM_ARRAY_PRE_THREAD];
static mut MEMORY_SIZE: Vec<AtomicUsize> = Vec::new(); //NUM_TASKS * NUM_ARRAY_PRE_THREAD] = [AtomicUsize::new(0); NUM_TASKS * NUM_ARRAY_PRE_THREAD];

static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

pub fn multi_thread_test() {
    srand(2333);
    println!("Multi thread memory allocation test begin.");
    let t0 = std::time::Instant::now();
    unsafe {
        MEMORY_POOL.clear();
        MEMORY_SIZE.clear();
        for _ in 0..NUM_TASKS * NUM_ARRAY_PRE_THREAD {
            MEMORY_POOL.push(AtomicUsize::new(0));
            MEMORY_SIZE.push(AtomicUsize::new(0));
        }
    }

    for _ in 0..MUN_TURN {
        // alloc memory and free half (only free the memory allocated by itself)
        FINISHED_TASKS.store(0, Ordering::Relaxed);
        for i in 0..NUM_TASKS {
            thread::spawn(move || {
                unsafe {
                    let tid = i;
                    for j in 0..NUM_ARRAY_PRE_THREAD {
                        let size = (1_usize << (rand_u32() % 12)) + (1_usize << (rand_u32() % 12));
                        let idx = j * NUM_TASKS + tid;
                        if let Ok(ptr) =
                            GLOBAL_ALLOCATOR.alloc(Layout::from_size_align_unchecked(size, 8))
                        {
                            //println!("successfully alloc: {:#?} {:#x} {:#?}", idx,ptr,size);
                            MEMORY_POOL[idx].store(ptr, Ordering::Relaxed);
                            MEMORY_SIZE[idx].store(size, Ordering::Relaxed);
                        } else {
                            panic!("multi thread test: alloc err,");
                        }
                    }

                    for j in (NUM_ARRAY_PRE_THREAD >> 1)..NUM_ARRAY_PRE_THREAD {
                        let idx = j * NUM_TASKS + tid;
                        let addr = MEMORY_POOL[idx].load(Ordering::Relaxed);
                        let size = MEMORY_SIZE[idx].load(Ordering::Relaxed);
                        //println!("dealloc: {:#?} {:#x} {:#?}", idx,addr,size);
                        GLOBAL_ALLOCATOR
                            .dealloc(addr as _, Layout::from_size_align_unchecked(size, 8));
                        MEMORY_POOL[idx].store(0_usize, Ordering::Relaxed);
                        MEMORY_SIZE[idx].store(0_usize, Ordering::Relaxed);
                    }
                }
                FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
            });
        }

        while FINISHED_TASKS.load(Ordering::Relaxed) < NUM_TASKS {
            thread::sleep(Duration::from_millis(10));
        }

        // realloc memory and free all
        FINISHED_TASKS.store(0, Ordering::Relaxed);
        for i in 0..NUM_TASKS {
            thread::spawn(move || {
                unsafe {
                    let tid = i;
                    for j in 0..(NUM_ARRAY_PRE_THREAD >> 1) {
                        let size = (1_usize << (rand_u32() % 12)) + (1_usize << (rand_u32() % 12));
                        let idx = NUM_TASKS * NUM_ARRAY_PRE_THREAD / 2
                            + tid * NUM_ARRAY_PRE_THREAD / 2
                            + j;
                        if let Ok(ptr) =
                            GLOBAL_ALLOCATOR.alloc(Layout::from_size_align_unchecked(size, 8))
                        {
                            MEMORY_POOL[idx].store(ptr, Ordering::Relaxed);
                            MEMORY_SIZE[idx].store(size, Ordering::Relaxed);
                        } else {
                            panic!("multi thread test: alloc err,");
                        }
                    }

                    for j in 0..NUM_ARRAY_PRE_THREAD {
                        let idx = j * NUM_TASKS + tid;
                        while MEMORY_SIZE[idx].load(Ordering::Relaxed) == 0 {
                            thread::sleep(Duration::from_millis(10));
                        }
                        let addr = MEMORY_POOL[idx].load(Ordering::Relaxed);
                        let size = MEMORY_SIZE[idx].load(Ordering::Relaxed);
                        GLOBAL_ALLOCATOR
                            .dealloc(addr as _, Layout::from_size_align_unchecked(size, 8));
                        MEMORY_POOL[idx].store(0_usize, Ordering::Relaxed);
                        MEMORY_SIZE[idx].store(0_usize, Ordering::Relaxed);
                    }
                }
                FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
            });
        }
        while FINISHED_TASKS.load(Ordering::Relaxed) < NUM_TASKS {
            thread::sleep(Duration::from_millis(10));
        }
    }
    let t1 = std::time::Instant::now();
    println!("time: {:#?}", t1 - t0);
    println!("Align alloc test OK!");
    println!("Multi thread memory allocation test OK!");
}

/*
#[test]
fn system_alloc_test() {
    unsafe {
        GLOBAL_ALLOCATOR.init_heap();
    }
    call_back_test(233);
    println!("Running memory tests...");

    println!("system alloc test:");
    unsafe {
        GLOBAL_ALLOCATOR.init_system();
    }
    align_test();
    basic_test();
    mi_test();
    malloc_large_test();
    glibc_bench_test();
    multi_thread_test();
    multi_thread_c_test();
    println!("system test passed!");
    println!("*****************************");
}
*/

#[test]
fn test_start() {
    srand(2333);
    unsafe {
        GLOBAL_ALLOCATOR.init_heap();
    }
    call_back_test(233);
    println!("Running memory tests...");

    println!("system alloc test:");
    unsafe {
        GLOBAL_ALLOCATOR.init_system();
    }
    align_test();
    basic_test();
    mi_test();
    malloc_large_test();
    glibc_bench_test();
    multi_thread_test();
    multi_thread_c_test();
    println!("system test passed!");
    println!("*****************************");

    println!("tlsf_rust alloc test:");
    unsafe {
        GLOBAL_ALLOCATOR.init_tlsf_rust();
    }
    align_test();
    basic_test();
    mi_test();
    malloc_large_test();
    glibc_bench_test();
    //multi_thread_test();
    //multi_thread_c_test();
    println!("tlsf_rust alloc test passed!");
    println!("*****************************");
    unsafe {
        GLOBAL_ALLOCATOR.init_system();
    }

    println!("first fit alloc test:");
    unsafe {
        GLOBAL_ALLOCATOR.init_basic("first_fit");
    }
    //align_test();
    basic_test();
    mi_test();
    //malloc_large_test();
    //glibc_bench_test();
    //multi_thread_test();
    //multi_thread_c_test();
    println!("first fit alloc test passed!");
    println!("*****************************");
    unsafe {
        GLOBAL_ALLOCATOR.init_system();
    }

    println!("best fit alloc test:");
    unsafe {
        GLOBAL_ALLOCATOR.init_basic("best_fit");
    }
    //align_test();
    basic_test();
    mi_test();
    //malloc_large_test();
    //glibc_bench_test();
    //multi_thread_test();
    //multi_thread_c_test();
    println!("best fit alloc test passed!");
    println!("*****************************");
    unsafe {
        GLOBAL_ALLOCATOR.init_system();
    }

    println!("worst fit alloc test:");
    unsafe {
        GLOBAL_ALLOCATOR.init_basic("worst_fit");
    }
    //align_test();
    basic_test();
    mi_test();
    //malloc_large_test();
    //glibc_bench_test();
    //multi_thread_test();
    //multi_thread_c_test();
    println!("worst fit alloc test passed!");
    println!("*****************************");
    unsafe {
        GLOBAL_ALLOCATOR.init_system();
    }

    println!("buddy alloc test:");
    unsafe {
        GLOBAL_ALLOCATOR.init_buddy();
    }
    //align_test();
    basic_test();
    mi_test();
    //malloc_large_test();
    //glibc_bench_test();
    //multi_thread_test();
    //multi_thread_c_test();
    println!("buddy alloc test passed!");
    println!("*****************************");
    unsafe {
        GLOBAL_ALLOCATOR.init_system();
    }

    println!("slab alloc test:");
    unsafe {
        GLOBAL_ALLOCATOR.init_slab();
    }
    //align_test();
    basic_test();
    mi_test();
    //malloc_large_test();
    //glibc_bench_test();
    //multi_thread_test();
    //multi_thread_c_test();
    println!("slab alloc test passed!");
    println!("*****************************");
    unsafe {
        GLOBAL_ALLOCATOR.init_system();
    }

    println!("tlsf_c alloc test:");
    unsafe {
        GLOBAL_ALLOCATOR.init_tlsf_c();
    }
    align_test();
    basic_test();
    mi_test();
    malloc_large_test();
    glibc_bench_test();
    //multi_thread_test();
    //multi_thread_c_test();
    println!("tlsf_c alloc test passed!");
    println!("*****************************");
    unsafe {
        GLOBAL_ALLOCATOR.init_system();
    }

    println!("Memory tests run OK!");
}

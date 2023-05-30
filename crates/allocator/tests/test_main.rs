#![feature(ptr_alignment_type)]
mod global_allocator;
use allocator_test;
use allocator_test::*;
use core::panic;
use core::sync::atomic::{AtomicUsize, Ordering};
use global_allocator::GLOBAL_ALLOCATOR;
use std::alloc::Layout;
use std::thread;
use std::time::Duration;
use std::vec::Vec;

pub fn basic_test() {
    println!("Basic alloc test begin...");
    let t0 = std::time::Instant::now();
    allocator_test::basic_test();
    let t1 = std::time::Instant::now();
    println!("time: {:#?}", t1 - t0);
    println!("Basic alloc test OK!");
}
pub fn mi_test() {
    println!("Mi alloc test begin...");
    let t0 = std::time::Instant::now();
    allocator_test::mi_test();
    let t1 = std::time::Instant::now();
    println!("time: {:#?}", t1 - t0);
    println!("Mi alloc test OK!");
}
pub fn malloc_large_test() {
    println!("Malloc large test begin...");
    let t0 = std::time::Instant::now();
    allocator_test::malloc_large_test();
    let t1 = std::time::Instant::now();
    println!("time: {:#?}", t1 - t0);
    println!("Malloc large test OK!");
}
pub fn glibc_bench_test() {
    println!("Glibc bench test begin...");
    let t0 = std::time::Instant::now();
    allocator_test::glibc_bench_test();
    let t1 = std::time::Instant::now();
    println!("time: {:#?}", t1 - t0);
    println!("Glibc bench test OK!");
}
pub fn multi_thread_c_test() {
    println!("Multi thread C test begin...");
    let t0 = std::time::Instant::now();
    allocator_test::multi_thread_c_test();
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

/// align test
pub fn align_test() {
    println!("Align alloc test begin...");
    let t0 = std::time::Instant::now();
    allocator_test::align_test();
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

//#[test]
fn system_alloc_test() {
    srand(2333);
    unsafe {
        GLOBAL_ALLOCATOR.init_heap();
    }
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

//#[test]
fn tlsf_rust_alloc_test() {
    srand(2333);
    unsafe {
        GLOBAL_ALLOCATOR.init_heap();
    }
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
}

//#[test]
fn tlsf_c_alloc_test() {
    srand(2333);
    unsafe {
        GLOBAL_ALLOCATOR.init_heap();
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
}

//#[test]
fn first_fit_alloc_test() {
    srand(2333);
    unsafe {
        GLOBAL_ALLOCATOR.init_heap();
    }
    println!("first fit alloc test:");
    unsafe {
        GLOBAL_ALLOCATOR.init_basic("first_fit");
    }
    //align_test();
    basic_test();
    mi_test();
    malloc_large_test();
    glibc_bench_test();
    //multi_thread_test();
    //multi_thread_c_test();
    println!("first fit alloc test passed!");
    println!("*****************************");
    unsafe {
        GLOBAL_ALLOCATOR.init_system();
    }
}

//#[test]
fn best_fit_alloc_test() {
    srand(2333);
    unsafe {
        GLOBAL_ALLOCATOR.init_heap();
    }
    println!("best fit alloc test:");
    unsafe {
        GLOBAL_ALLOCATOR.init_basic("best_fit");
    }
    //align_test();
    basic_test();
    mi_test();
    malloc_large_test();
    glibc_bench_test();
    //multi_thread_test();
    //multi_thread_c_test();
    println!("best fit alloc test passed!");
    println!("*****************************");
    unsafe {
        GLOBAL_ALLOCATOR.init_system();
    }
}

//#[test]
fn worst_fit_alloc_test() {
    srand(2333);
    unsafe {
        GLOBAL_ALLOCATOR.init_heap();
    }
    println!("worst fit alloc test:");
    unsafe {
        GLOBAL_ALLOCATOR.init_basic("worst_fit");
    }
    //align_test();
    basic_test();
    mi_test();
    malloc_large_test();
    glibc_bench_test();
    //multi_thread_test();
    //multi_thread_c_test();
    println!("worst fit alloc test passed!");
    println!("*****************************");
    unsafe {
        GLOBAL_ALLOCATOR.init_system();
    }
}

//#[test]
fn buddy_fit_alloc_test() {
    srand(2333);
    unsafe {
        GLOBAL_ALLOCATOR.init_heap();
    }
    println!("buddy alloc test:");
    unsafe {
        GLOBAL_ALLOCATOR.init_buddy();
    }
    //align_test();
    basic_test();
    mi_test();
    malloc_large_test();
    glibc_bench_test();
    //multi_thread_test();
    //multi_thread_c_test();
    println!("buddy alloc test passed!");
    println!("*****************************");
    unsafe {
        GLOBAL_ALLOCATOR.init_system();
    }
}

//#[test]
fn slab_alloc_test() {
    srand(2333);
    unsafe {
        GLOBAL_ALLOCATOR.init_heap();
    }
    println!("slab alloc test:");
    unsafe {
        GLOBAL_ALLOCATOR.init_slab();
    }
    //align_test();
    basic_test();
    mi_test();
    malloc_large_test();
    glibc_bench_test();
    //multi_thread_test();
    //multi_thread_c_test();
    println!("slab alloc test passed!");
    println!("*****************************");
    unsafe {
        GLOBAL_ALLOCATOR.init_system();
    }
}

#[test]
fn test_start() {
    system_alloc_test();
    tlsf_rust_alloc_test();
    tlsf_c_alloc_test();
    first_fit_alloc_test();
    best_fit_alloc_test();
    worst_fit_alloc_test();
    buddy_fit_alloc_test();
    slab_alloc_test();
}

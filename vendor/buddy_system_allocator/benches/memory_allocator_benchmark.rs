#[macro_use]
extern crate alloc;
#[macro_use]
extern crate ctor;

use std::sync::Arc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use alloc::alloc::GlobalAlloc;
use alloc::alloc::Layout;
use buddy_system_allocator::LockedHeap;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const SMALL_SIZE: usize = 8;
const LARGE_SIZE: usize = 1024 * 1024; // 1M
const ALIGN: usize = 8;

/// Alloc small object
#[inline]
pub fn small_alloc<const ORDER: usize>(heap: &LockedHeap<ORDER>) {
    let layout = unsafe { Layout::from_size_align_unchecked(SMALL_SIZE, ALIGN) };
    unsafe {
        let addr = heap.alloc(layout);
        heap.dealloc(addr, layout);
    }
}

/// Alloc large object
#[inline]
pub fn large_alloc<const ORDER: usize>(heap: &LockedHeap<ORDER>) {
    let layout = unsafe { Layout::from_size_align_unchecked(LARGE_SIZE, ALIGN) };
    unsafe {
        let addr = heap.alloc(layout);
        heap.dealloc(addr, layout);
    }
}

/// Multithreads alloc random sizes of object
#[inline]
pub fn mutil_thread_random_size<const ORDER: usize>(heap: &'static LockedHeap<ORDER>) {
    const THREAD_SIZE: usize = 10;

    use rand::prelude::*;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;

    let mut threads = Vec::with_capacity(THREAD_SIZE);
    let alloc = Arc::new(heap);
    for i in 0..THREAD_SIZE {
        let prethread_alloc = alloc.clone();
        let handle = thread::spawn(move || {
            // generate a random size of object use seed `i` to ensure the fixed
            // result of each turn
            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(i as u64);
            // generate a random object size in range of [SMALL_SIZE ..= LARGE_SIZE]
            let layout = unsafe {
                Layout::from_size_align_unchecked(rng.gen_range(SMALL_SIZE..=LARGE_SIZE), ALIGN)
            };
            let addr = unsafe { prethread_alloc.alloc(layout) };

            // sleep for a while
            sleep(Duration::from_nanos((THREAD_SIZE - i) as u64));

            unsafe { prethread_alloc.dealloc(addr, layout) }
        });
        threads.push(handle);
    }
    drop(alloc);

    for t in threads {
        t.join().unwrap();
    }
}

/// Multithread benchmark inspired by **Hoard** benchmark
///
/// Warning: This benchmark generally needs long time to finish
///
/// ----------------------------------------------------------------------
/// Hoard: A Fast, Scalable, and Memory-Efficient Allocator
///       for Shared-Memory Multiprocessors
/// Contact author: Emery Berger, http://www.cs.utexas.edu/users/emery
//
/// Copyright (c) 1998-2000, The University of Texas at Austin.
///
/// This library is free software; you can redistribute it and/or modify
/// it under the terms of the GNU Library General Public License as
/// published by the Free Software Foundation, http://www.fsf.org.
///
/// This library is distributed in the hope that it will be useful, but
/// WITHOUT ANY WARRANTY; without even the implied warranty of
/// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
/// Library General Public License for more details.
/// ----------------------------------------------------------------------
///
#[inline]
pub fn thread_test() {
    const N_ITERATIONS: usize = 50;
    const N_OBJECTS: usize = 30000;
    const N_THREADS: usize = 10;
    const OBJECT_SIZE: usize = 1;

    #[derive(Clone)]
    struct Foo {
        pub a: i32,
        pub b: i32,
    }

    let mut threads = Vec::with_capacity(N_THREADS);

    for _i in 0..N_THREADS {
        let handle = thread::spawn(move || {
            // let a = new Foo * [nobjects / nthreads];
            let mut a = Vec::with_capacity(N_OBJECTS / N_THREADS);
            for j in 0..N_ITERATIONS {
                // inner object:
                // a[i] = new Foo[objSize];
                for k in 0..(N_OBJECTS / N_THREADS) {
                    a.push(vec![
                        Foo {
                            a: k as i32,
                            b: j as i32
                        };
                        OBJECT_SIZE
                    ]);

                    // in order to prevent optimization delete allocation directly
                    // FIXME: don't know whether it works or not
                    a[k][0].a += a[k][0].b;
                }
            }
            // auto drop here
        });
        threads.push(handle);
    }

    for t in threads {
        t.join().unwrap();
    }
}

const ORDER: usize = 32;
const MACHINE_ALIGN: usize = core::mem::size_of::<usize>();
/// for now 128M is needed
/// TODO: reduce memory use
const KERNEL_HEAP_SIZE: usize = 128 * 1024 * 1024;
const HEAP_BLOCK: usize = KERNEL_HEAP_SIZE / MACHINE_ALIGN;
static mut HEAP: [usize; HEAP_BLOCK] = [0; HEAP_BLOCK];

/// Use `LockedHeap` as global allocator
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<ORDER> = LockedHeap::<ORDER>::new();

/// Init heap
///
/// We need `ctor` here because benchmark is running behind the std enviroment,
/// which means std will do some initialization before execute `fn main()`.
/// However, our memory allocator must be init in runtime(use linkedlist, which
/// can not be evaluated in compile time). And in the initialization phase, heap
/// memory is needed.
///
/// So the solution in this dilemma is to run `fn init_heap()` in initialization phase
/// rather than in `fn main()`. We need `ctor` to do this.
#[ctor]
fn init_heap() {
    let heap_start = unsafe { HEAP.as_ptr() as usize };
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(heap_start, HEAP_BLOCK * MACHINE_ALIGN);
    }
}

/// Entry of benchmarks
pub fn criterion_benchmark(c: &mut Criterion) {
    // run benchmark
    c.bench_function("small alloc", |b| {
        b.iter(|| small_alloc(black_box(&HEAP_ALLOCATOR)))
    });
    c.bench_function("large alloc", |b| {
        b.iter(|| large_alloc(black_box(&HEAP_ALLOCATOR)))
    });
    c.bench_function("mutil thread random size", |b| {
        b.iter(|| mutil_thread_random_size(black_box(&HEAP_ALLOCATOR)))
    });
    c.bench_function("threadtest", |b| b.iter(|| thread_test()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

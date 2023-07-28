#![feature(allocator_api)]
#![feature(btreemap_alloc)]

mod utils;

use std::alloc::Allocator;
use std::collections::BTreeMap;
use std::io::Write;

use allocator::{AllocatorRc, BuddyByteAllocator, SlabByteAllocator, TlsfByteAllocator};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{rngs::SmallRng, seq::SliceRandom, RngCore, SeedableRng};

use self::utils::MemoryPool;

const POOL_SIZE: usize = 1024 * 1024 * 128;

fn vec_push(n: usize, alloc: &(impl Allocator + Clone)) {
    let mut v: Vec<u32, _> = Vec::new_in(alloc.clone());
    for _ in 0..n {
        v.push(0xdead_beef);
    }
    drop(v);
}

fn vec_rand_free(n: usize, blk_size: usize, alloc: &(impl Allocator + Clone)) {
    let mut v = Vec::new_in(alloc.clone());
    for _ in 0..n {
        let block = Vec::<u64, _>::with_capacity_in(blk_size, alloc.clone());
        v.push(block);
    }

    let mut rng = SmallRng::seed_from_u64(0xdead_beef);
    let mut index = Vec::with_capacity_in(n, alloc.clone());
    for i in 0..n {
        index.push(i);
    }
    index.shuffle(&mut rng);

    for i in index {
        v[i] = Vec::new_in(alloc.clone());
    }
    drop(v);
}

fn btree_map(n: usize, alloc: &(impl Allocator + Clone)) {
    let mut rng = SmallRng::seed_from_u64(0xdead_beef);
    let mut m = BTreeMap::new_in(alloc.clone());
    for _ in 0..n {
        if rng.next_u32() % 5 == 0 && !m.is_empty() {
            m.pop_first();
        } else {
            let value = rng.next_u32();
            let mut key = Vec::new_in(alloc.clone());
            write!(&mut key, "key_{value}").unwrap();
            m.insert(key, value);
        }
    }
    m.clear();
    drop(m);
}

fn bench(c: &mut Criterion, alloc_name: &str, alloc: impl Allocator + Clone) {
    let mut g = c.benchmark_group(alloc_name);
    g.bench_function("vec_push_3M", |b| {
        b.iter(|| vec_push(black_box(3_000_000), &alloc));
    });
    g.sample_size(10);
    g.bench_function("vec_rand_free_25K_64", |b| {
        b.iter(|| vec_rand_free(black_box(25_000), black_box(64), &alloc));
    });
    g.bench_function("vec_rand_free_7500_520", |b| {
        b.iter(|| vec_rand_free(black_box(7_500), black_box(520), &alloc));
    });
    g.bench_function("btree_map_50K", |b| {
        b.iter(|| btree_map(black_box(50_000), &alloc));
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut pool = MemoryPool::new(POOL_SIZE);
    bench(c, "system", std::alloc::System);
    bench(
        c,
        "tlsf",
        AllocatorRc::new(TlsfByteAllocator::new(), pool.as_slice()),
    );
    bench(
        c,
        "slab",
        AllocatorRc::new(SlabByteAllocator::new(), pool.as_slice()),
    );
    bench(
        c,
        "buddy",
        AllocatorRc::new(BuddyByteAllocator::new(), pool.as_slice()),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

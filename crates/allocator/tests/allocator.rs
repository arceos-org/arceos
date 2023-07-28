#![feature(btreemap_alloc)]
#![feature(allocator_api)]
#![feature(strict_provenance)]

use std::alloc::{Allocator, Layout};
use std::collections::BTreeMap;
use std::io::Write;

use allocator::{AllocatorRc, BuddyByteAllocator, SlabByteAllocator, TlsfByteAllocator};
use rand::{prelude::SliceRandom, Rng};

const POOL_SIZE: usize = 1024 * 1024 * 128;

fn test_vec(n: usize, alloc: &(impl Allocator + Clone)) {
    let mut v = Vec::with_capacity_in(n, alloc.clone());
    for _ in 0..n {
        v.push(rand::random::<u32>());
    }
    v.sort();
    for i in 0..n - 1 {
        assert!(v[i] <= v[i + 1]);
    }
}

fn test_vec2(n: usize, blk_size: usize, alloc: &(impl Allocator + Clone)) {
    let mut v = Vec::new_in(alloc.clone());
    for _ in 0..n {
        let block = Vec::<u64, _>::with_capacity_in(blk_size, alloc.clone());
        v.push(block);
    }

    let mut index = Vec::with_capacity_in(n, alloc.clone());
    for i in 0..n {
        index.push(i);
    }
    index.shuffle(&mut rand::thread_rng());

    for i in index {
        v[i] = Vec::new_in(alloc.clone())
    }
}

fn test_btree_map(n: usize, alloc: &(impl Allocator + Clone)) {
    let mut m = BTreeMap::new_in(alloc.clone());
    for _ in 0..n {
        if rand::random::<u32>() % 5 == 0 && !m.is_empty() {
            m.pop_first();
        } else {
            let value = rand::random::<u32>();
            let mut key = Vec::new_in(alloc.clone());
            write!(&mut key, "key_{value}").unwrap();
            m.insert(key, value);
        }
    }
    for (k, v) in m.iter() {
        let key = std::str::from_utf8(k)
            .unwrap()
            .strip_prefix("key_")
            .unwrap();
        assert_eq!(key.parse::<u32>().unwrap(), *v);
    }
}

pub fn test_alignment(n: usize, alloc: &(impl Allocator + Clone)) {
    let mut rng = rand::thread_rng();
    let mut blocks = vec![];
    for _ in 0..n {
        if rng.gen_ratio(2, 3) || blocks.len() == 0 {
            // insert a block
            let size =
                ((1 << rng.gen_range(0..16)) as f32 * rng.gen_range(1.0..2.0)).round() as usize;
            let align = 1 << rng.gen_range(0..8);
            let layout = Layout::from_size_align(size, align).unwrap();
            let ptr = alloc.allocate(layout).unwrap();
            blocks.push((ptr, layout));
        } else {
            // delete a block
            let idx = rng.gen_range(0..blocks.len());
            let blk = blocks.swap_remove(idx);
            unsafe { alloc.deallocate(blk.0.cast(), blk.1) };
        }
    }
    for blk in blocks {
        unsafe { alloc.deallocate(blk.0.cast(), blk.1) };
    }
}

fn run_test(f: impl FnOnce(&mut [u8])) {
    let layout = Layout::from_size_align(POOL_SIZE, 4096).unwrap();
    let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
    let pool = unsafe { core::slice::from_raw_parts_mut(ptr, POOL_SIZE) };

    f(pool);

    unsafe { std::alloc::dealloc(ptr, layout) };
}

#[test]
fn system_alloc() {
    run_test(|_pool| {
        let alloc = std::alloc::System;
        test_alignment(50, &alloc);
        test_vec(3_000_000, &alloc);
        test_vec2(30_000, 64, &alloc);
        test_vec2(7_500, 520, &alloc);
        test_btree_map(50_000, &alloc);
    })
}

#[test]
fn buddy_alloc() {
    run_test(|pool| {
        let alloc = AllocatorRc::new(BuddyByteAllocator::new(), pool);
        test_alignment(50, &alloc);
        test_vec(3_000_000, &alloc);
        test_vec2(30_000, 64, &alloc);
        test_vec2(7_500, 520, &alloc);
        test_btree_map(50_000, &alloc);
    })
}

#[test]
fn slab_alloc() {
    run_test(|pool| {
        let alloc = AllocatorRc::new(SlabByteAllocator::new(), pool);
        test_alignment(50, &alloc);
        test_vec(3_000_000, &alloc);
        test_vec2(30_000, 64, &alloc);
        test_vec2(7_500, 520, &alloc);
        test_btree_map(50_000, &alloc);
    })
}

#[test]
fn tlsf_alloc() {
    run_test(|pool| {
        let alloc = AllocatorRc::new(TlsfByteAllocator::new(), pool);
        test_alignment(50, &alloc);
        test_vec(3_000_000, &alloc);
        test_vec2(30_000, 64, &alloc);
        test_vec2(7_500, 520, &alloc);
        test_btree_map(50_000, &alloc);
    })
}

#![no_std]
#![no_main]

#[macro_use]
extern crate axruntime;
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

fn rand() -> u64 {
    use core::sync::atomic::{AtomicU64, Ordering::SeqCst};
    static SEED: AtomicU64 = AtomicU64::new(0xdeaf_beef);
    let new_seed = SEED.load(SeqCst) * 6364136223846793005 + 1;
    SEED.store(new_seed, SeqCst);
    new_seed >> 33
}

fn test_vec() {
    const N: usize = 1_000_000;
    let mut v = Vec::with_capacity(N);
    for _ in 0..N {
        v.push(rand());
    }
    v.sort();
    for i in 0..N - 1 {
        assert!(v[i] <= v[i + 1]);
    }
    println!("test_vec() OK!");
}

fn test_btree_map() {
    const N: usize = 10_000;
    let mut m = BTreeMap::new();
    for _ in 0..N {
        let value = rand();
        let key = alloc::format!("key_{value}");
        m.insert(key, value);
    }
    for (k, v) in m.iter() {
        if let Some(k) = k.strip_prefix("key_") {
            assert_eq!(k.parse::<u64>().unwrap(), *v);
        }
    }
    println!("test_btree_map() OK!");
}

#[no_mangle]
fn main() {
    println!("Running memory tests...");
    test_vec();
    test_btree_map();
    println!("Memory tests run OK!");
}

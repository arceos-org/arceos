#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use libax::rand;

fn test_vec() {
    const N: usize = 1_000_000;
    let mut v = Vec::with_capacity(N);
    for _ in 0..N {
        v.push(rand::rand_u32());
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
        let value = rand::rand_u32();
        let key = alloc::format!("key_{value}");
        m.insert(key, value);
    }
    for (k, v) in m.iter() {
        if let Some(k) = k.strip_prefix("key_") {
            assert_eq!(k.parse::<u32>().unwrap(), *v);
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

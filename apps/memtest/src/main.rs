#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use rand::{rngs::SmallRng, RngCore, SeedableRng};
use std::collections::BTreeMap;
use std::vec::Vec;

fn test_vec(rng: &mut impl RngCore) {
    const N: usize = 3_000_000;
    let mut v = Vec::with_capacity(N);
    for _ in 0..N {
        v.push(rng.next_u32());
    }
    v.sort();
    for i in 0..N - 1 {
        assert!(v[i] <= v[i + 1]);
    }
    println!("test_vec() OK!");
}

fn test_btree_map(rng: &mut impl RngCore) {
    const N: usize = 50_000;
    let mut m = BTreeMap::new();
    for _ in 0..N {
        let value = rng.next_u32();
        let key = format!("key_{value}");
        m.insert(key, value);
    }
    for (k, v) in m.iter() {
        if let Some(k) = k.strip_prefix("key_") {
            assert_eq!(k.parse::<u32>().unwrap(), *v);
        }
    }
    println!("test_btree_map() OK!");
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("Running memory tests...");

    let mut rng = SmallRng::seed_from_u64(0xdead_beef);
    test_vec(&mut rng);
    test_btree_map(&mut rng);

    println!("Memory tests run OK!");
}

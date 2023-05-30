use alloc::collections::BTreeMap;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering::SeqCst};
use log::debug;

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

/// test of vec
pub fn test_vec(n: usize) {
    debug!("test_vec() begin...");
    let mut v = Vec::new();
    for _ in 0..n {
        v.push(rand_u32());
    }
    debug!("test_vec() OK!");
}

/// test of btreemap
pub fn test_btree_map(n: usize) {
    debug!("test_btree_map() begin...");
    let mut m = BTreeMap::new();
    for _ in 0..n {
        if rand_usize() % 5 == 0 && !m.is_empty() {
            m.pop_first();
        } else {
            let value = rand_usize();
            let key = format!("key_{value}");
            m.insert(key, value);
        }
    }
    for (k, v) in m.iter() {
        if let Some(k) = k.strip_prefix("key_") {
            assert_eq!(k.parse::<usize>().unwrap(), *v);
        }
    }
    debug!("test_btree_map() OK!");
}

/// another test of vec
pub fn test_vec_2(n: usize, m: usize) {
    debug!("test_vec2() begin...");
    let mut v: Vec<Vec<usize>> = Vec::new();
    for _ in 0..n {
        let mut tmp: Vec<usize> = Vec::with_capacity(m);
        for _ in 0..m {
            tmp.push(rand_usize());
        }
        tmp.sort();
        for j in 0..m - 1 {
            assert!(tmp[j] <= tmp[j + 1]);
        }
        v.push(tmp);
    }

    let mut p: Vec<usize> = Vec::with_capacity(n);
    for i in 0..n {
        p.push(i);
    }

    for i in 1..n {
        let o: usize = rand_usize() % (i + 1);
        p.swap(i, o);
    }
    for o in p.iter().take(n) {
        let tmp: Vec<usize> = Vec::new();
        v[*o] = tmp;
    }
    debug!("test_vec2() OK!");
}

/// third test of vec
pub fn test_vec_3(n: usize, k1: usize, k2: usize) {
    debug!("test_vec3() begin...");
    let mut v: Vec<Vec<usize>> = Vec::new();
    for i in 0..n * 4 {
        let nw = match i >= n * 2 {
            true => k1,
            false => match i % 2 {
                0 => k1,
                _ => k2,
            },
        };
        v.push(Vec::with_capacity(nw));
        for _ in 0..nw {
            v[i].push(rand_usize());
        }
    }
    for (i, o) in v.iter_mut().enumerate().take(n * 4) {
        if i % 2 == 1 {
            let tmp: Vec<usize> = Vec::new();
            *o = tmp;
        }
    }
    for i in 0..n {
        let nw = k2;
        v.push(Vec::with_capacity(nw));
        for _ in 0..nw {
            v[4 * n + i].push(rand_usize());
        }
    }
    debug!("test_vec3() OK!");
}

/// basic test
pub fn basic_test() {
    test_vec(3000000);
    test_vec_2(30000, 64);
    test_vec_2(7500, 520);
    test_btree_map(50000);
    test_vec_3(10000, 32, 64);
}

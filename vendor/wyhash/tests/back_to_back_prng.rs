extern crate wyhash;
use wyhash::{wyrng, WyRng};
extern crate rand_core;
use rand_core::{RngCore, SeedableRng};

#[test]
fn wyrng_test() {
    for (i, original) in ORIGINAL_PRNG.iter().enumerate() {
        let mut seed = i as u64;
        assert_eq!(*original, wyrng(&mut seed));
    }
}

#[test]
fn rngcore_trait_next_64() {
    let mut rng = WyRng::default();
    for original in ORIGINAL_PRNG_SEQ.iter() {
        assert_eq!(*original, rng.next_u64());
    }
}

#[test]
fn rngcore_trait_next_32() {
    let mut rng = WyRng::default();
    for original in ORIGINAL_PRNG_SEQ.iter() {
        assert_eq!((*original) as u32, rng.next_u32());
    }
}

#[test]
fn seedablerng_trait() {
    for (i, original) in ORIGINAL_PRNG.iter().enumerate() {
        let seed = [i as u8, 0, 0, 0, 0, 0, 0, 0];
        let mut rng = WyRng::from_seed(seed);
        assert_eq!(*original, rng.next_u64());
    }
}

#[test]
fn seedablerng_trait_seed_from_u64() {
    for (i, original) in ORIGINAL_PRNG.iter().enumerate() {
        let mut rng = WyRng::seed_from_u64(i as u64);
        assert_eq!(*original, rng.next_u64());
    }
}

fn read64_le(data: &[u8]) -> [u64; 10] {
    let mut packed = [0; 10];
    for (i, chunk) in data.chunks(8).enumerate() {
        for (j, d) in chunk.iter().enumerate() {
            packed[i] |= u64::from(*d) << (j * 8);
        }
    }
    packed
}

fn check_prng_seq(data: &[u8]) {
    let packed = read64_le(&data);
    for (original, current) in ORIGINAL_PRNG_SEQ.iter().zip(&packed) {
        assert_eq!(*original, *current);
    }
}

#[test]
fn rngcore_trait_fill_bytes() {
    let mut rng = WyRng::default();
    let mut data = [0; 80];
    rng.fill_bytes(&mut data);

    check_prng_seq(&data);
}

#[test]
fn rngcore_trait_try_fill_bytes() {
    let mut rng = WyRng::default();
    let mut data = [0; 80];
    rng.try_fill_bytes(&mut data).expect("Failed to fill bytes");

    check_prng_seq(&data);
}

// Results from the cannonical C implementation
#[allow(clippy::unreadable_literal)]
const ORIGINAL_PRNG: [u64; 10] = [
    0x111cb3a78f59a58e,
    0xcdef1695e1f8ed2c,
    0xa4eed0248024f5f6,
    0x3e99a772750dcbe,
    0xfae94589c79d2703,
    0xac19123cacd229cc,
    0xb18dc4f431e3006,
    0xe21b87e1e24a18c1,
    0x591b413082f6638b,
    0x35d5241efb19a892,
];

// Results from the cannonical C implementation
#[allow(clippy::unreadable_literal)]
const ORIGINAL_PRNG_SEQ: [u64; 10] = [
    0x111cb3a78f59a58e,
    0xceabd938ff4e856d,
    0x61fb51318f47d2a4,
    0x78bd03c491909760,
    0x7c003d7fb14820de,
    0x8769964729356b1f,
    0xe214284dc87f9829,
    0x29a283ebb1b295a2,
    0xf4e11accbc44be57,
    0x9a108fea1a03ac0a,
];

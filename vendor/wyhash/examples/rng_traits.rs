// Example using the `rand_core::RngCore` and `rand_core::SeedableRng` interface.
// (Same as using `rand::Rng` and `rand::SeedableRng`)

use rand_core::{RngCore, SeedableRng};
use wyhash::WyRng;

fn main() {
    // Seeds are 8-byte long.
    let seed = [0, 1, 2, 3, 4, 5, 6, 7];
    let mut rng1 = WyRng::from_seed(seed);
    assert_eq!(0xd730_1357_74c6_ae31, rng1.next_u64());

    // Alternatively you can also use this convenience method:
    let mut rng2 = WyRng::seed_from_u64(3);
    assert_eq!(0x3e9_9a77_2750_dcbe, rng2.next_u64());
}

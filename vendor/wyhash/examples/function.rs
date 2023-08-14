// Example of using the free functions.

use wyhash::{wyhash, wyrng};

fn main() {
    assert_eq!(0xb0f9_4152_0b1a_d95d, wyhash(&[0, 1, 2], 3));

    let mut rng_seed = 1;
    let random_number = wyrng(&mut rng_seed);
    assert_eq!(0xcdef_1695_e1f8_ed2c, random_number);
    assert_eq!(0xa076_1d64_78bd_6430, rng_seed);
}

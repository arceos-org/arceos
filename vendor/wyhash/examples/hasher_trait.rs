// Example using the `core::hash::Hasher` interface.
// (Same as using `std::hash::Hasher`)

use core::hash::Hasher;
use wyhash::WyHash;

fn main() {
    let mut hasher = WyHash::with_seed(3);
    hasher.write(&[0, 1, 2]);
    assert_eq!(0xb0f9_4152_0b1a_d95d, hasher.finish());
}

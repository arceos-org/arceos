extern crate core;
extern crate wyhash;
use core::hash::Hasher;
use wyhash::WyHash;

#[test]
fn default_constructed() {
    let mut hasher = WyHash::default();
    hasher.write(&[0]);
    assert_eq!(0x8c73_a8ab_4659_6ae4, hasher.finish());
}

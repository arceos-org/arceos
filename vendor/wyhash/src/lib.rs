//! Rust implementation of the [wyhash algorithm by Wang Yi][original].
//!
//! The hashing algorithm passes SMHasher and the random number generator
//! passes BigCrush and practrand.
//! As of now it is the fastest algorithm in the SMHasher benchmark
//! (faster than t1ha and XXH3).
//! See [here][original].
//!
//! Furthermore, this algorithm is solid, simple, portable (does not need
//! hardware support, can be used in `no_std` environments) and has
//! no dependencies.
//!
//! The generated hashes are equal (see tests) as of the version stated
//! [here][original-version] although the speed varies
//! ([PRs are welcome][issue-tracker]).
//!
//! [issue-tracker]: https://github.com/eldruin/wyhash-rs/issues
//! [original]: https://github.com/wangyi-fudan/wyhash
//! [original-version]: https://github.com/eldruin/wyhash-rs/blob/master/original/CMakeLists.txt
//!
//! ## Usage (see also examples folder)
//!
//! For the hashing function you can use either the free function or the
//! `Hasher` trait.
//!
//! ### `wyhash` function usage
//!
//! ```
//! use wyhash::wyhash;
//!
//! let data = [0, 1, 2];
//! let seed = 3;
//! let hash = wyhash(&data, seed);
//!
//! assert_eq!(0xb0f9_4152_0b1a_d95d, hash);
//! ```
//!
//! ### `Hasher` trait usage
//!
//! You can also use `std::hash::Hasher`, it is the same.
//!
//! ```
//! use core::hash::Hasher;
//! use wyhash::WyHash;
//!
//! let mut hasher = WyHash::with_seed(3);
//! hasher.write(&[0, 1, 2]);
//!
//! assert_eq!(0xb0f9_4152_0b1a_d95d, hasher.finish());
//! ```
//!
//! ### `wyrng` function usage
//!
//! Note that the seed parameter is updated so that it is possible to
//! generate a sequence of random numbers.
//!
//! ```
//! use wyhash::wyrng;
//!
//! let mut seed = 3;
//! let random_number = wyrng(&mut seed);
//!
//! assert_eq!(0x3e9_9a77_2750_dcbe, random_number);
//! assert_eq!(0xa0761d6478bd6432, seed);
//! ```
//!
//! ### `RngCore` trait usage
//!
//! You can also use `rand::Rng`, it is the same.
//!
//! ```
//! use rand_core::RngCore;
//! use wyhash::WyRng;
//!
//! let mut rng = WyRng::default();
//! assert_eq!(0x111c_b3a7_8f59_a58e, rng.next_u64());
//! ```
//!
//! ### `SeedableRng` trait usage
//!
//! You can also use `rand::SeedableRng`, it is the same.
//!
//! ```
//! use rand_core::{SeedableRng, RngCore};
//! use wyhash::WyRng;
//!
//! // Seeds are 8-byte long.
//! let seed = [0, 1, 2, 3, 4, 5, 6, 7];
//! let mut rng1 = WyRng::from_seed(seed);
//! assert_eq!(0xd730_1357_74c6_ae31, rng1.next_u64());
//!
//! // Alternatively you can also use this convenience method:
//! let mut rng2 = WyRng::seed_from_u64(3);
//! assert_eq!(0x3e9_9a77_2750_dcbe, rng2.next_u64());
//! ```

#![doc(html_root_url = "https://docs.rs/wyhash/0.5.0")]
#![no_std]
#![deny(missing_docs, unsafe_code)]

mod functions;
pub use crate::functions::{wyhash, wyrng};

mod traits;
pub use crate::traits::{WyHash, WyRng};

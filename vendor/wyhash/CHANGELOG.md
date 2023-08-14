# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

...

## [0.5.0] - 2020-12-17

### Changed
- [breaking-change] Updated `rand_core` dependency from version `0.5` to version
  `0.6`. This led to an increase in the Minimum Supported Rust Version from
  `1.32.0` to `1.36.0`.

## [0.4.2] - 2020-11-07
### Added
- Implement `Clone` for `WyHash` and `WyRng`. Thanks to @Restioson.
- "`hasher`" keyword for better discoverability. Thanks to @tkaitchuck.
  See: [rust-lang/rust#77996](https://github.com/rust-lang/rust/pull/77996)

## [0.4.1] - 2020-06-28
### Fixed
- Formatting of MSRV section in Readme.

## [0.4.0] - 2020-06-28
### Changed
- [breaking-change] Updated `rand_core` dependency from version `0.4` to version
  `0.5`. This led to an increase in the Minimum Supported Rust Version from
  `1.31.0` to `1.32.0`.

## [0.3.0] - 2019-06-02
### Fixed
- [breaking-change] The random number generator now uses only the updated seed
  as the internal state instead of the last generated number. This leads to the
  free function `wyrng` function now receiving a mutable reference to the seed
  which will be used to represent the state, following the upstream interface.
  The `RngCore` and `SeedableRng` trait implementations for `WyRng` will return
  different numbers as in the last published version.
  See: https://github.com/eldruin/wyhash-rs/issues/1.

## [0.2.1] - 2019-03-30
### Added
- `rand_core::RngCore` and `rand_core::SeedableRng` trait implementations
  for the random number generator.
- MIT license

## [0.2.0] - 2019-03-23
### Added
- Added random number generation function.
- Added C++ program using the upstream library to generate the results used
  in the tests.

### Changed
- The standard library is not necessary any more. The hasher trait implemented
  now is `core::hash::Hasher`, which is equivalent to `std::hash::Hasher`.
  The code should continue to work without change but deactivating
  the default features for `no_std` compatibility is not necessary any more.

- The generated hashes have changed following the upstream project.

## 0.1.0 - 2019-03-11

This is the initial release to crates.io.

[Unreleased]: https://github.com/eldruin/wyhash-rs/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/eldruin/wyhash-rs/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/eldruin/wyhash-rs/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/eldruin/wyhash-rs/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/eldruin/wyhash-rs/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/eldruin/wyhash-rs/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/eldruin/wyhash-rs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/eldruin/wyhash-rs/compare/v0.1.0...v0.2.0

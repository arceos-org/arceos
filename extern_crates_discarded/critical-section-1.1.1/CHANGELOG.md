# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- (Add unreleased changes here)

## [v1.1.1] - 2022-09-13

- On the `std` implementation, panicking inside the `critical_section::with()` closure no longer accidentally leaves the critical section locked (#26).

## [v1.1.0] - 2022-08-17

- Added built-in critical section implementation using `std::sync::Mutex`, enabled by the `std` Cargo feature.
- MSRV changed to `1.54` when `std` feature is disabled, `1.63` when enabled.

## [v1.0.0] - 2022-08-10

- Improved docs.

## [v1.0.0-alpha.2] - 2022-07-28

- Change name of the `extern fn`s to avoid clash with critical-section 0.2.

## [v1.0.0-alpha.1] - 2022-07-28

Breaking changes:

- Removed all builtin impls. These are going to be provided by platform-support crates now.
- Renamed `custom_impl!` to `set_impl!`.
- RestoreState is now an opaque struct for the user, and a transparent `RawRestoreState` type alias for impl writers.
- RestoreState type is now configurable with Cargo features. Default is `()`. (previously it was fixed to `u8`.)
- Added own `CriticalSection` and `Mutex` types, instead of reexporting them from `bare_metal`.

## [v0.2.7] - 2022-04-08

- Add support for AVR targets.

## [v0.2.6] - 2022-04-02

- Improved docs.

## [v0.2.5] - 2021-11-02

- Fix `std` implementation to allow reentrant (nested) critical sections. This would previously deadlock.

## [v0.2.4] - 2021-09-24

- Add support for 32bit RISC-V targets.

## [v0.2.3] - 2021-09-13

- Use correct `#[vcfg]` for `wasm` targets.

## [v0.2.2] - 2021-09-13

- Added support for `wasm` targets.

## [v0.2.1] - 2021-05-11

- Added critical section implementation for `std`, based on a global Mutex.

## [v0.2.0] - 2021-05-10

- Breaking change: use `CriticalSection<'_>` instead of `&CriticalSection<'_>`

## v0.1.0 - 2021-05-10

- First release

[Unreleased]: https://github.com/rust-embedded/critical-section/compare/v1.1.1...HEAD
[v1.1.1]: https://github.com/rust-embedded/critical-section/compare/v1.1.0...v1.1.1
[v1.1.0]: https://github.com/rust-embedded/critical-section/compare/v1.0.0...v1.1.0
[v1.0.0]: https://github.com/rust-embedded/critical-section/compare/v1.0.0-alpha.2...v1.0.0
[v1.0.0-alpha.2]: https://github.com/rust-embedded/critical-section/compare/v1.0.0-alpha.1...v1.0.0-alpha.2
[v1.0.0-alpha.1]: https://github.com/rust-embedded/critical-section/compare/v0.2.7...v1.0.0-alpha.1
[v0.2.7]: https://github.com/rust-embedded/critical-section/compare/v0.2.6...v0.2.7
[v0.2.6]: https://github.com/rust-embedded/critical-section/compare/v0.2.5...v0.2.6
[v0.2.5]: https://github.com/rust-embedded/critical-section/compare/v0.2.4...v0.2.5
[v0.2.4]: https://github.com/rust-embedded/critical-section/compare/v0.2.3...v0.2.4
[v0.2.3]: https://github.com/rust-embedded/critical-section/compare/v0.2.2...v0.2.3
[v0.2.2]: https://github.com/rust-embedded/critical-section/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/rust-embedded/critical-section/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/rust-embedded/critical-section/compare/v0.1.0...v0.2.0
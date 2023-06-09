# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Modified

### Fixed

## [0.0.4] - 2022-10-10

### Added

- Various convenient functions to `SbiRet` structure
- Add documents on whole `sbi-rt` crate to coply with `deny(missing_docs)`
- Feature `legacy` to gate legacy SBI extension

### Modified

- Rename `SbiRet::ok` to `SbiRet::success`
- Rename `SbiSpecVersion` to struct `Version` 

## [0.0.3] - 2022-10-06

### Added

- deps: static_assertions
  check implementations during compilation, and provide an item list for developers
- denied: warnings and unsafe code
- a github workflow to check building
- `SbiSpecVersion` type defination for sbi base

### Modified

- rename `GET_SPEC_VERSION` to `GET_SBI_SPEC_VERSION`
- rename `impl_id::IMPL_XXX` to `impl_id::XXX`

### Removed

- default target to RISC-V

## [0.0.2] - 2022-07-21

### Added

- A changelog to this project

### Modified

- Lift build target limit; now this crate would build on targets other than RISC-V

## [0.0.1] - 2022-07-11

This is the first release of sbi-spec crate. This crate includes definition of RISC-V Supervisor Binary Interface (SBI) including structures and constants.

### Added

- Adapt to SBI specification version 1.0.0 ratified

[Unreleased]: https://github.com/rustsbi/sbi-spec/compare/v0.0.4...HEAD
[0.0.4]: https://github.com/rustsbi/sbi-spec/compare/v0.0.3...v0.0.4
[0.0.3]: https://github.com/rustsbi/sbi-spec/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/rustsbi/sbi-spec/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/rustsbi/sbi-spec/releases/tag/v0.0.1

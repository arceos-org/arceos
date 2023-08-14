# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [v0.2.0] - 2018-05-10

### Changed

- [breaking-change] `const-fn` is no longer a default feature (i.e. a feature that's enabled by
  default). The consequence is that this crate now compiles on 1.27 (beta) by default, and opting
  into `const-fn` requires nightly.

## [v0.1.2] - 2018-04-25

### Added

- an opt-out "const-fn" Cargo feature. Disabling this feature removes all `const` constructors and
  makes this crate compilable on stable.

## [v0.1.1] - 2017-05-30

### Added

- support for aligned slices

## v0.1.0 - 2017-05-29

- Initial release

[Unreleased]: https://github.com/japaric/aligned/compare/v0.2.0...HEAD
[v0.2.0]: https://github.com/japaric/aligned/compare/v0.1.2...v0.2.0
[v0.1.2]: https://github.com/japaric/aligned/compare/v0.1.1...v0.1.2
[v0.1.1]: https://github.com/japaric/aligned/compare/v0.1.0...v0.1.1

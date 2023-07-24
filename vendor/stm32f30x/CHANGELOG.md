# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [v0.8.0] - 2018-12-07

### Changed

- [breaking-change] re-generated using svd2rust v0.14.0. The `interrupt` macro
  has become an attribute.

## [v0.7.1] - 2018-09-07

### Added

- Support for cortex-m-rt v0.6.x

## [v0.7.0] - 2018-05-12

### Changed

- [breaking-change] re-generated using svd2rust v0.13.0. This crate now compiles on stable.

- [breaking-change] the minor versions of the cortex-m, cortex-m-rt and bare-metal dependencies have
  been increased.

## [v0.6.2] - 2018-05-07

### Changed

- Made some bitfields in I2C registers safe to write to (using `<WriteConstraint>`)

## [v0.6.1] - 2018-05-06

### Changed

- Re-generated using svd2rust v0.12.1

## [v0.6.0] - 2018-01-15

### Changed

- [breaking-change] Re-generated using svd2rust v0.12.0
- writes to some USART and I2C registers are now safe

### Added

- A USART3EN field to RCC.APB1ENR

### Fixed

- The reset value of some GPIO and SPI registers.

## [v0.5.1] - 2017-08-01

### Changed

- Re-generated using svd2rust v0.11.3

### Fixed

- Overrides of interrupt handles were being ignored if LTO was not enabled.

## [v0.5.0] - 2017-07-31 - YANKED

- Re-generate usign svd2rust v0.11.2

## [v0.4.1] - 2017-05-08

- Re-generate usign svd2rust v0.7.2

## [v0.4.0] - 2017-04-25

### Changed

- [breaking-change] Re-generated using svd2rust v0.7.0. NVIC and FPU API
  changed.

### Added

- API for the rest of the core peripherals. This API is just a re-export of the
  cortex-m one.

## [v0.3.2] - 2017-04-23

### Added

- enumeratedValues to some GPIO, TIM and RCC bitfields
- writeConstraint to some TIM bitfields

### Changed

- Re-generate using svd2rust v0.6.2

## [v0.3.1] - 2017-04-15

### Changed

- [breaking-change] Re-generate using svd2rust v0.6.1

## [v0.3.0] - 2017-04-11

### Changed

- [breaking-change] Re-generate using svd2rust v0.6.0

## [v0.2.0] - 2017-03-27

### Changed

- [breaking-change] Re-generate using svd2rust v0.5.0

## v0.1.0 - 2017-03-12

- Initial release

[Unreleased]: https://github.com/japaric/stm32f30x/compare/v0.8.0...HEAD
[v0.8.0]: https://github.com/japaric/stm32f30x/compare/v0.7.1...v0.8.0
[v0.7.1]: https://github.com/japaric/stm32f30x/compare/v0.7.0...v0.7.1
[v0.7.0]: https://github.com/japaric/stm32f30x/compare/v0.6.2...v0.7.0
[v0.6.2]: https://github.com/japaric/stm32f30x/compare/v0.6.1...v0.6.2
[v0.6.1]: https://github.com/japaric/stm32f30x/compare/v0.6.0...v0.6.1
[v0.6.0]: https://github.com/japaric/stm32f30x/compare/v0.5.1...v0.6.0
[v0.5.1]: https://github.com/japaric/stm32f30x/compare/v0.5.0...v0.5.1
[v0.5.0]: https://github.com/japaric/stm32f30x/compare/v0.4.1...v0.5.0
[v0.4.1]: https://github.com/japaric/stm32f30x/compare/v0.4.0...v0.4.1
[v0.4.0]: https://github.com/japaric/stm32f30x/compare/v0.3.2...v0.4.0
[v0.3.2]: https://github.com/japaric/stm32f30x/compare/v0.3.1...v0.3.2
[v0.3.1]: https://github.com/japaric/stm32f30x/compare/v0.3.0...v0.3.1
[v0.3.0]: https://github.com/japaric/stm32f30x/compare/v0.2.0...v0.3.0
[v0.2.0]: https://github.com/japaric/stm32f30x/compare/v0.1.0...v0.2.0

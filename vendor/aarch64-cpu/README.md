[![crates.io](https://img.shields.io/crates/d/aarch64-cpu.svg)](https://crates.io/crates/aarch64-cpu)
[![crates.io](https://img.shields.io/crates/v/aarch64-cpu.svg)](https://crates.io/crates/aarch64-cpu)

# aarch64-cpu

Low level access to processors using the AArch64 execution state.

## Usage

Please note that for using this crate's [register definitions](src/registers) (as provided by
`aarch64_cpu::registers::*`), you need to also include
[`tock-registers`](https://crates.io/crates/tock-registers) in your project. This is because the
`interface` traits provided by `tock-registers` are implemented by this crate. You should include
the same version of `tock-registers` as is being used by this crate to ensure sane
interoperatbility.

For example, in the following snippet, `X.Y.Z` should be the same version of `tock-registers` that
is mentioned in `aarch64-cpu`'s [`Cargo.toml`](Cargo.toml#L23).

```toml
[package]
name = "Your embedded project"

# Some parts omitted for brevity.

[dependencies]
tock-registers = "X.Y.Z"
aarch64-cpu = "A.B.C"       # <-- Includes tock-registers itself.
```

### Example

Check out https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials for usage examples. Listed
below is a snippet of `rust-raspberrypi-OS-tutorials`'s early boot code.

```rust
use aarch64_cpu::{asm, registers::*};
use tock_registers::interfaces::Writeable; // <-- Trait needed to use `write()` and `set()`.

// Some parts omitted for brevity.

unsafe fn prepare_el2_to_el1_transition(
    virt_boot_core_stack_end_exclusive_addr: u64,
    virt_kernel_init_addr: u64,
) {
    // Enable timer counter registers for EL1.
    CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);

    // No offset for reading the counters.
    CNTVOFF_EL2.set(0);

    // Set EL1 execution state to AArch64.
    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);

    // Set up a simulated exception return.
    SPSR_EL2.write(
        SPSR_EL2::D::Masked
            + SPSR_EL2::A::Masked
            + SPSR_EL2::I::Masked
            + SPSR_EL2::F::Masked
            + SPSR_EL2::M::EL1h,
    );
}
```

## Disclaimer

Descriptive comments in the source files are taken from the
[ARM Architecture Reference Manual ARMv8, for ARMv8-A architecture profile](https://static.docs.arm.com/ddi0487/ca/DDI0487C_a_armv8_arm.pdf?_ga=2.266626254.1122218691.1534883460-1326731866.1530967873).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

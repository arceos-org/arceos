//! Platform-specific constants and parameters for [ArceOS].
//!
//! Currently supported platforms (specify by cargo features):
//!
//! - `platform-pc-x86`: Standard PC with x86_64 ISA.
//! - `platform-qemu-virt-riscv`: QEMU virt machine with RISC-V ISA.
//! - `platform-qemu-virt-aarch64`: QEMU virt machine with AArch64 ISA.
//! - `dummy`: If none of the above platform is selected, the dummy platform
//!    will be used. In this platform, most of the constants are dummy values.
//!    This platform is mainly used for [cargo test].
//!
//! [ArceOS]: https://github.com/rcore-os/arceos
//! [cargo test]: https://doc.rust-lang.org/cargo/guide/tests.html

#![no_std]

#[rustfmt::skip]
macro_rules! platform_config {
    ($plat_ident: ident, $config_path: literal) => {
        #[cfg($plat_ident)]
        #[rustfmt::skip]
        mod config {
            include!(concat!(env!("OUT_DIR"), "/", $config_path));
        }
    };
}

platform_config!(pc_x86, "config_pc_x86.rs");
platform_config!(qemu_virt_riscv, "config_qemu_virt_riscv.rs");
platform_config!(qemu_virt_aarch64, "config_qemu_virt_aarch64.rs");
platform_config!(raspi4_aarch64, "config_raspi4_aarch64.rs");
platform_config!(dummy, "config_dummy.rs");

pub use config::*;

/// End address of the whole physical memory.
pub const PHYS_MEMORY_END: usize = PHYS_MEMORY_BASE + PHYS_MEMORY_SIZE;

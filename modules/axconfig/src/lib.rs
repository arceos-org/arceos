//! Platform-specific constants and parameters for
//! [ArceOS](https://github.com/rcore-os/arceos).
//!
//! Currently supported platforms (corresponding cargo features):
//!
//! - `platform-qemu-virt-riscv`: QEMU virt machine with RISC-V ISA.
//! - `platform-qemu-virt-aarch64`: QEMU virt machine with AArch64 ISA.

#![no_std]

cfg_if::cfg_if! {
    if #[cfg(feature = "platform-qemu-virt-riscv")] {
        #[rustfmt::skip]
        #[path = "config_qemu_virt_riscv.rs"]
        mod config;
    } else if #[cfg(feature = "platform-qemu-virt-aarch64")] {
        #[rustfmt::skip]
        #[path = "config_qemu_virt_aarch64.rs"]
        mod config;
    } else {
        #[rustfmt::skip]
        #[path = "config_dummy.rs"]
        mod config;
    }
}

pub use config::*;

/// End address of the whole physical memory.
pub const PHYS_MEMORY_END: usize = PHYS_MEMORY_BASE + PHYS_MEMORY_SIZE;

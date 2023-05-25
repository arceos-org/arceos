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

pub const PHYS_MEMORY_END: usize = PHYS_MEMORY_BASE + PHYS_MEMORY_SIZE;

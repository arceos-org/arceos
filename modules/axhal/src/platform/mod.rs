//! Platform-specific operations.

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")]{
        mod aarch64_common;
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(target_arch = "x86_64", platform_family = "x86-pc"))] {
        mod x86_pc;
        pub use self::x86_pc::*;
    } else if #[cfg(all(target_arch = "riscv64", platform_family = "riscv64-qemu-virt"))] {
        mod riscv64_qemu_virt;
        pub use self::riscv64_qemu_virt::*;
    } else if #[cfg(all(target_arch = "aarch64", platform_family = "aarch64-qemu-virt"))] {
        mod aarch64_qemu_virt;
        pub use self::aarch64_qemu_virt::*;
    } else if #[cfg(all(target_arch = "aarch64", platform_family = "aarch64-raspi"))] {
        mod aarch64_raspi;
        pub use self::aarch64_raspi::*;
    } else {
        mod dummy;
        pub use self::dummy::*;
    }
}

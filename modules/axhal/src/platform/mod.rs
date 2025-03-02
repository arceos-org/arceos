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
    } else if #[cfg(all(target_arch = "aarch64", platform_family = "aarch64-bsta1000b"))] {
        mod aarch64_bsta1000b;
        pub use self::aarch64_bsta1000b::*;
    } else if #[cfg(all(target_arch = "aarch64", platform_family = "aarch64-phytium-pi"))] {
        mod aarch64_phytium_pi;
        pub use self::aarch64_phytium_pi::*;
    } else if #[cfg(all(target_arch = "loongarch64", platform_family = "loongarch64-qemu-virt"))] {
        mod loongarch64_qemu_virt;
        pub use self::loongarch64_qemu_virt::*;
    } else {
        mod dummy;
        pub use self::dummy::*;
    }
}

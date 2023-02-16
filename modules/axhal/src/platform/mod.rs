cfg_if::cfg_if! {
    if #[cfg(feature = "platform-qemu-virt-riscv")] {
        mod qemu_virt_riscv;
        pub use self::qemu_virt_riscv::*;
    } else if #[cfg(feature = "platform-qemu-virt-aarch64")]{
        mod qemu_virt_aarch64;
        pub use self::qemu_virt_aarch64::*;
    } else {
        mod dummy;
        pub use self::dummy::*;
    }
}

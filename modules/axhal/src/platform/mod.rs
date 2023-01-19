cfg_if::cfg_if! {
    if #[cfg(feature = "platform-qemu-virt-riscv")] {
        mod qemu_virt_riscv;
        pub use self::qemu_virt_riscv::*;
    }
}

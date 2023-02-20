cfg_if::cfg_if! {
    if #[cfg(any(target_arch = "x86_64"))] {
        pub mod x86_64;
    } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
        pub mod riscv;
    } else if #[cfg(target_arch = "aarch64")] {
        pub mod aarch64;
    }
}

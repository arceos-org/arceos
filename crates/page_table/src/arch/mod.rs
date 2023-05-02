#[cfg(any(target_arch = "x86_64", doc))]
pub mod x86_64;

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64", doc))]
pub mod riscv;

#[cfg(any(target_arch = "aarch64", doc))]
pub mod aarch64;

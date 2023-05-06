#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[doc(cfg(any(target_arch = "riscv32", target_arch = "riscv64")))]
pub mod riscv;

// TODO: `#[cfg(any(target_arch = "aarch64", doc))]` does not work.
#[doc(cfg(target_arch = "aarch64"))]
pub mod aarch64;

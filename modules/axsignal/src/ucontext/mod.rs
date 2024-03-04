
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
mod riscv64;

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
pub use riscv64::*;

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

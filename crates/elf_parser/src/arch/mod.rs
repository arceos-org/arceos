//! Architecture-specific types and operations.

use memory_addr::VirtAddr;
#[derive(Debug)]
/// To describe the relocation pair in the ELF
pub struct RelocatePair {
    pub src: VirtAddr, // the source address of the relocation
    pub dst: VirtAddr, // the destination address of the relocation
    pub count: usize,  // the set of bits affected by this relocation
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        mod x86_64;
        pub use x86_64::*;
    } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
        mod riscv;
        pub use riscv::*;
    } else if #[cfg(target_arch = "aarch64")]{
        mod aarch64;
        pub use self::aarch64::*;
    }
}

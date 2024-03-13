//! 记录该模块使用到的系统调用 id
//!
//!
#[cfg(any(
    target_arch = "riscv32",
    target_arch = "riscv64",
    target_arch = "aarch64"
))]
numeric_enum_macro::numeric_enum! {
#[repr(usize)]
#[allow(non_camel_case_types)]
#[allow(missing_docs)]
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum MemSyscallId {
    // mem
    SHMGET = 194,
    SHMCTL = 195,
    SHMAT = 196,
    BRK = 214,
    MUNMAP = 215,
    MMAP = 222,
    MSYNC = 227,
    MPROTECT = 226,
    MEMBARRIER = 283,
}
}

#[cfg(target_arch = "x86_64")]
numeric_enum_macro::numeric_enum! {
    #[repr(usize)]
    #[allow(non_camel_case_types)]
    #[allow(missing_docs)]
    #[derive(Eq, PartialEq, Debug, Copy, Clone)]
    pub enum MemSyscallId {
        // mem
        SHMGET = 29,
        SHMCTL = 31,
        SHMAT = 30,
        BRK = 12,
        MUNMAP = 11,
        MMAP = 9,
        MSYNC = 26,
        MPROTECT = 10,
        MEMBARRIER = 324,
    }
}

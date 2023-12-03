//! 记录该模块使用到的系统调用 id
//!
//!
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

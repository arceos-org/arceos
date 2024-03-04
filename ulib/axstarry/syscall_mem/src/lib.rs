#![cfg_attr(all(not(test), not(doc)), no_std)]
use syscall_utils::SyscallResult;

mod imp;

mod mem_syscall_id;

pub use mem_syscall_id::MemSyscallId;
use mem_syscall_id::MemSyscallId::*;

use imp::*;
pub fn mem_syscall(syscall_id: mem_syscall_id::MemSyscallId, args: [usize; 6]) -> SyscallResult {
    match syscall_id {
        BRK => syscall_brk(args),
        MUNMAP => syscall_munmap(args),
        #[cfg(feature = "fs")]
        MMAP => syscall_mmap(args),
        MSYNC => syscall_msync(args),
        MPROTECT => syscall_mprotect(args),
        MEMBARRIER => Ok(0),
        SHMGET => syscall_shmget(args),
        SHMCTL => Ok(0),
        SHMAT => syscall_shmat(args),
        #[allow(unused)]
        _ => {
            panic!("Invalid Syscall Id: {:?}!", syscall_id);
        }
    }
}

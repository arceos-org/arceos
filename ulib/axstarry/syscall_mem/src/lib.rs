#![cfg_attr(all(not(test), not(doc)), no_std)]
use syscall_utils::{MMAPFlags, SyscallResult, MMAPPROT};

mod imp;

mod mem_syscall_id;

pub use mem_syscall_id::MemSyscallId;
use mem_syscall_id::MemSyscallId::*;

use imp::*;
pub fn mem_syscall(syscall_id: mem_syscall_id::MemSyscallId, args: [usize; 6]) -> SyscallResult {
    match syscall_id {
        BRK => syscall_brk(args[0] as usize),
        MUNMAP => syscall_munmap(args[0], args[1]),
        #[cfg(feature = "fs")]
        MMAP => syscall_mmap(
            args[0],
            args[1],
            MMAPPROT::from_bits_truncate(args[2] as u32),
            MMAPFlags::from_bits_truncate(args[3] as u32),
            args[4] as i32,
            args[5],
        ),
        MSYNC => syscall_msync(args[0], args[1]),
        MPROTECT => syscall_mprotect(
            args[0] as usize,
            args[1] as usize,
            MMAPPROT::from_bits_truncate(args[2] as u32),
        ),
        MEMBARRIER => Ok(0),
        SHMGET => syscall_shmget(args[0] as i32, args[1], args[2] as i32),
        SHMCTL => Ok(0),
        SHMAT => syscall_shmat(args[0] as i32, args[1], args[2] as i32),
        #[allow(unused)]
        _ => {
            panic!("Invalid Syscall Id: {:?}!", syscall_id);
        }
    }
}

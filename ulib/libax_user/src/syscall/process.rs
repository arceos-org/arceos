//! syscalls about processes
use axio::Read;
use syscall_number::{SYS_EXEC, SYS_FORK, SYS_WAIT};

use crate::{io::File, syscall};
extern crate alloc;
use alloc::vec::Vec;

/// `fork` another process with the same memory contents and file tables.
pub fn fork() -> isize {
    syscall(SYS_FORK, [0, 0, 0, 0, 0, 0])
}

/// `wait` for process to stop
pub fn wait(pid: usize, ret: &mut i32) -> usize {
    syscall(SYS_WAIT, [pid, ret as *mut i32 as usize, 0, 0, 0, 0]) as usize
}

/// run the program
pub fn exec(path: &str) -> isize {
    if let Ok(data) = File::open(path).and_then(|mut file| {
        let mut data: Vec<u8> = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(data)
    }) {
        syscall(SYS_EXEC, [data.as_ptr() as usize, data.len(), 0, 0, 0, 0])
    } else {
        -1
    }
}

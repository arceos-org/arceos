use core::slice;

use axerrno::{AxResult, from_ret_code};
use syscall_number::{io::OpenFlags, SYS_READ, SYS_OPEN, SYS_CLOSE};

use super::sys_number::SYS_WRITE;

pub fn write(fd: usize, s: &[u8]) -> isize {
    crate::syscall(SYS_WRITE, [fd, s.as_ptr() as usize, s.len(), 0, 0, 0])
}

pub fn read(fd: usize, s: &mut [u8]) -> isize {
    crate::syscall(SYS_READ, [fd, s.as_ptr() as usize, s.len(), 0, 0, 0])
}

pub fn open(path: &str, permission: OpenFlags) -> isize {
    crate::syscall(SYS_OPEN, [path.as_ptr() as usize, path.len(), permission.bits(), 0, 0, 0])
}

pub fn close(fd: usize) -> isize {
    crate::syscall(SYS_CLOSE, [fd, 0, 0, 0, 0, 0])
}

pub struct File {
    fd: usize,       
}
impl File {
    pub fn open(path: &str) -> AxResult<Self> {
        from_ret_code(open(path, OpenFlags::empty()))
            .map(|fd| Self { fd })            
    }
    pub fn create(path: &str) -> AxResult<Self> {
        from_ret_code(open(path, OpenFlags::CREATE))
            .map(|fd| Self { fd })            
    }
    pub fn read(&self, s: &mut [u8]) -> AxResult<usize> {
        from_ret_code(read(self.fd, s))
    }
    pub fn read_data<T>(&self, s: &mut T) -> AxResult<usize> {
        from_ret_code(read(self.fd, unsafe {
            core::slice::from_raw_parts_mut(
                s as *mut T as *mut u8,
                core::mem::size_of::<T>()
            )
        }))
    }
    pub fn write(&self, s: &[u8]) -> AxResult<usize> {
        from_ret_code(write(self.fd, s))
    }
    pub fn write_data<T>(&self, s: &T) -> AxResult<usize> {
        from_ret_code(write(self.fd, unsafe {
            core::slice::from_raw_parts(
                s as *const T as *mut u8,
                core::mem::size_of::<T>()
            )
        }))
    }
}
impl Drop for File {
    fn drop(&mut self) {
        close(self.fd);
    }
}


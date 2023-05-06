use super::file_io::FileIO;
use axerrno::{AxError, AxResult};
use axhal::console::{getchar, write_bytes};
use axtask::yield_now;

/// stdin file for getting chars from console
pub struct Stdin;

/// stdout file for putting chars to console
pub struct Stdout;

/// stderr file for putting chars to console
pub struct Stderr;

impl FileIO for Stdin {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
    fn read(&self, _buf: &mut [u8]) -> AxResult<usize> {
        let ch: u8;
        loop {
            match getchar() {
                Some(c) => {
                    ch = c;
                    break;
                }
                None => {
                    yield_now();
                    continue;
                }
            }
        }
        unsafe {
            _buf.as_mut_ptr().write_volatile(ch);
        }
        Ok(1)
    }
    fn write(&self, _buf: &[u8]) -> AxResult<usize> {
        panic!("Cannot write to stdin!");
    }
    fn seek(&self, _pos: usize) -> AxResult<u64> {
        Err(AxError::Unsupported) // 如果没有实现seek, 则返回Unsupported
    }
}

impl FileIO for Stdout {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, _buf: &mut [u8]) -> AxResult<usize> {
        panic!("Cannot read from stdout!");
    }
    fn write(&self, _buf: &[u8]) -> AxResult<usize> {
        // for buffer in buf.iter() {
        //     print!("{}", core::str::from_utf8(buf).unwrap());
        // }
        write_bytes(_buf);
        Ok(_buf.len())
    }
    fn seek(&self, _pos: usize) -> AxResult<u64> {
        Err(AxError::Unsupported) // 如果没有实现seek, 则返回Unsupported
    }
}

impl FileIO for Stderr {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, _buf: &mut [u8]) -> AxResult<usize> {
        panic!("Cannot read from stdout!");
    }
    fn write(&self, _buf: &[u8]) -> AxResult<usize> {
        write_bytes(_buf);
        Ok(_buf.len())
    }
    fn seek(&self, _pos: usize) -> AxResult<u64> {
        Err(AxError::Unsupported) // 如果没有实现seek, 则返回Unsupported
    }
}

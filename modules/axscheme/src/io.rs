extern crate alloc;
use alloc::vec::Vec;
use axsync::{Mutex, MutexGuard};
use scheme::Scheme;

use super::KernelScheme;

pub struct Stdout {
    data: Mutex<Vec<u8>>,
}
impl Stdout {
    fn flush(&self, mut data: MutexGuard<Vec<u8>>) {
        use axhal::console::putchar;
        info!("Writing user content");
        for i in data.iter() {
            putchar(*i);
        }
        data.clear();
    }
    fn putchar(&self, c: u8) {
        let mut data = self.data.lock();
        data.push(c);
        if c == b'\n' {
            self.flush(data);
        }
    }
    pub fn new() -> Self {
        Stdout {
            data: Mutex::new(Vec::new()),
        }
    }
}
impl Scheme for Stdout {
    fn open(&self, _path: &str, _flags: usize, _uid: u32, _gid: u32) -> axerrno::AxResult<usize> {
        Ok(1)
    }
    fn write(&self, _id: usize, buf: &[u8]) -> axerrno::AxResult<usize> {
        for i in buf {
            self.putchar(*i)
        }
        Ok(buf.len())
    }
    fn close(&self, _id: usize) -> axerrno::AxResult<usize> {
        self.flush(self.data.lock());
        Ok(0)
    }
    fn fsync(&self, _id: usize) -> axerrno::AxResult<usize> {
        self.flush(self.data.lock());
        Ok(0)
    }
}
impl KernelScheme for Stdout {}
pub struct Stdin;
impl Scheme for Stdin {
    fn open(&self, _path: &str, _flags: usize, _uid: u32, _gid: u32) -> axerrno::AxResult<usize> {
        Ok(1)
    }
    fn read(&self, _id: usize, buf: &mut [u8]) -> axerrno::AxResult<usize> {
        for i in buf.iter_mut() {
            *i = loop {
                match axhal::console::getchar() {
                    Some(c) => break c,
                    None => axtask::yield_now(),
                }
            }
        }
        Ok(buf.len())
    }
    fn close(&self, _id: usize) -> axerrno::AxResult<usize> {
        Ok(0)
    }
}
impl KernelScheme for Stdin {}

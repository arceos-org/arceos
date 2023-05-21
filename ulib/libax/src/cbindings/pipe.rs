use super::{ctypes, fd_table::Filelike};
use crate::sync::Mutex;
use crate::thread::yield_now;
use alloc::sync::Arc;
use axerrno::{LinuxError, LinuxResult};
use core::ffi::c_int;

#[derive(Copy, Clone, PartialEq)]
enum RingBufferStatus {
    Full,
    Empty,
    Normal,
}

const RING_BUFFER_SIZE: usize = 256;

pub struct PipeRingBuffer {
    arr: [u8; RING_BUFFER_SIZE],
    head: usize,
    tail: usize,
    status: RingBufferStatus,
}

impl PipeRingBuffer {
    pub const fn new() -> Self {
        Self {
            arr: [0; RING_BUFFER_SIZE],
            head: 0,
            tail: 0,
            status: RingBufferStatus::Empty,
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.status = RingBufferStatus::Normal;
        self.arr[self.tail] = byte;
        self.tail = (self.tail + 1) % RING_BUFFER_SIZE;
        if self.tail == self.head {
            self.status = RingBufferStatus::Full;
        }
    }

    pub fn read_byte(&mut self) -> u8 {
        self.status = RingBufferStatus::Normal;
        let c = self.arr[self.head];
        self.head = (self.head + 1) % RING_BUFFER_SIZE;
        if self.head == self.tail {
            self.status = RingBufferStatus::Empty;
        }
        c
    }

    /// Get the length of remaining data in the buffer
    pub const fn available_read(&self) -> usize {
        if matches!(self.status, RingBufferStatus::Empty) {
            0
        } else if self.tail > self.head {
            self.tail - self.head
        } else {
            self.tail + RING_BUFFER_SIZE - self.head
        }
    }

    /// Get the length of remaining space in the buffer
    pub const fn available_write(&self) -> usize {
        if matches!(self.status, RingBufferStatus::Full) {
            0
        } else {
            RING_BUFFER_SIZE - self.available_read()
        }
    }
}

pub struct Pipe {
    readable: bool,
    buffer: Arc<Mutex<PipeRingBuffer>>,
}

impl Pipe {
    pub fn new() -> (Arc<Mutex<Self>>, Arc<Mutex<Self>>) {
        let buffer = Arc::new(Mutex::new(PipeRingBuffer::new()));
        let read_end = Arc::new(Mutex::new(Pipe {
            readable: true,
            buffer: buffer.clone(),
        }));
        let write_end = Arc::new(Mutex::new(Pipe {
            readable: false,
            buffer,
        }));
        (read_end, write_end)
    }

    pub const fn readable(&self) -> bool {
        self.readable
    }

    pub const fn writable(&self) -> bool {
        !self.readable
    }

    pub fn write_end_close(&self) -> bool {
        Arc::strong_count(&self.buffer) == 1
    }
}

impl Pipe {
    pub fn read(&mut self, buf: &mut [u8]) -> LinuxResult<usize> {
        if !self.readable() {
            return Err(LinuxError::EPERM);
        }
        let mut read_size = 0usize;
        let max_len = buf.len();
        loop {
            let mut ring_buffer = self.buffer.lock();
            let loop_read = ring_buffer.available_read();
            if loop_read == 0 {
                if self.write_end_close() {
                    return Ok(read_size);
                }
                drop(ring_buffer);
                // Data not ready, wait for write end
                yield_now();
                continue;
            }
            for _ in 0..loop_read {
                if read_size == max_len {
                    return Ok(read_size);
                }
                buf[read_size] = ring_buffer.read_byte();
                read_size += 1;
            }
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> LinuxResult<usize> {
        if !self.writable() {
            return Err(LinuxError::EPERM);
        }
        let mut write_size = 0usize;
        let max_len = buf.len();
        loop {
            let mut ring_buffer = self.buffer.lock();
            let loop_write = ring_buffer.available_write();
            if loop_write == 0 {
                drop(ring_buffer);
                // Buffer is full, wait for read end to consume
                yield_now();
                continue;
            }
            for _ in 0..loop_write {
                if write_size == max_len {
                    return Ok(write_size);
                }
                ring_buffer.write_byte(buf[write_size]);
                write_size += 1;
            }
        }
    }
}

pub(super) fn stat_pipe(_pipe: &Pipe) -> LinuxResult<ctypes::stat> {
    let st_mode = 0o10000 | 0o666u32;
    Ok(ctypes::stat {
        st_ino: 1,
        st_nlink: 1,
        st_mode,
        st_blksize: 512,
        ..Default::default()
    })
}

/// Create a pipe
///
/// Return 0 if succeed
#[no_mangle]
pub unsafe extern "C" fn ax_pipe(fd1: *mut c_int, fd2: *mut c_int) -> c_int {
    ax_call_body!(ax_pipe, {
        let (pipe1, pipe2) = Pipe::new();
        let read_fd = Filelike::from_pipe(pipe1)
            .add_to_fd_table()
            .ok_or(LinuxError::EPIPE)?;
        let write_fd = Filelike::from_pipe(pipe2)
            .add_to_fd_table()
            .ok_or(LinuxError::EPIPE)?;
        unsafe {
            *fd1 = read_fd as c_int;
            *fd2 = write_fd as c_int;
        }
        Ok(0)
    })
}

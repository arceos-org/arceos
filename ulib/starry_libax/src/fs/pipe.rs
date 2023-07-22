use super::FileIOType;
extern crate alloc;
use alloc::sync::{Arc, Weak};
use axerrno::{AxError, AxResult};
use axfs::monolithic_fs::{file_io::FileExt, FileIO};
use axio::{Read, Seek, Write};
use axsync::Mutex;
use axtask::yield_now;
use log::{error, info, trace};
use spinlock::SpinNoIrq;

/// IPC pipe
pub struct Pipe {
    #[allow(unused)]
    readable: bool,
    #[allow(unused)]
    writable: bool,
    buffer: Arc<Mutex<PipeRingBuffer>>,
}

impl Pipe {
    /// create readable pipe
    pub fn read_end_with_buffer(buffer: Arc<Mutex<PipeRingBuffer>>) -> Self {
        Self {
            readable: true,
            writable: false,
            buffer,
        }
    }
    /// create writable pipe
    pub fn write_end_with_buffer(buffer: Arc<Mutex<PipeRingBuffer>>) -> Self {
        Self {
            readable: false,
            writable: true,
            buffer,
        }
    }
}

const RING_BUFFER_SIZE: usize = 0x40_000;

#[derive(Copy, Clone, PartialEq)]
enum RingBufferStatus {
    Full,
    Empty,
    Normal,
}

pub struct PipeRingBuffer {
    arr: [u8; RING_BUFFER_SIZE],
    head: usize,
    tail: usize,
    status: RingBufferStatus,
    write_end: Option<Weak<SpinNoIrq<Pipe>>>,
}

impl PipeRingBuffer {
    pub fn new() -> Self {
        Self {
            arr: [0; RING_BUFFER_SIZE],
            head: 0,
            tail: 0,
            status: RingBufferStatus::Empty,
            write_end: None,
        }
    }
    pub fn set_write_end(&mut self, write_end: &Arc<SpinNoIrq<Pipe>>) {
        self.write_end = Some(Arc::downgrade(write_end));
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
    pub fn available_read(&self) -> usize {
        if self.status == RingBufferStatus::Empty {
            0
        } else if self.tail > self.head {
            self.tail - self.head
        } else {
            self.tail + RING_BUFFER_SIZE - self.head
        }
    }
    pub fn available_write(&self) -> usize {
        if self.status == RingBufferStatus::Full {
            0
        } else {
            RING_BUFFER_SIZE - self.available_read()
        }
    }
    pub fn all_write_ends_closed(&self) -> bool {
        self.write_end.as_ref().unwrap().upgrade().is_none()
    }
}

/// Return (read_end, write_end)
pub fn make_pipe() -> (Arc<SpinNoIrq<Pipe>>, Arc<SpinNoIrq<Pipe>>) {
    trace!("kernel: make_pipe");
    let buffer = Arc::new(Mutex::new(PipeRingBuffer::new()));
    let read_end = Arc::new(SpinNoIrq::new(Pipe::read_end_with_buffer(buffer.clone())));
    let write_end = Arc::new(SpinNoIrq::new(Pipe::write_end_with_buffer(buffer.clone())));
    buffer.lock().set_write_end(&write_end);
    (read_end, write_end)
}

impl Read for Pipe {
    fn read(&mut self, buf: &mut [u8]) -> AxResult<usize> {
        trace!("kernel: Pipe::read");
        assert!(self.readable());
        let want_to_read = buf.len();
        let mut buf_iter = buf.iter_mut();
        let mut already_read = 0usize;
        // 防止pipe死循环
        let mut cnt = 0;
        loop {
            let mut ring_buffer = self.buffer.lock();
            let loop_read = ring_buffer.available_read();
            // info!("kernel: Pipe::read: already_read = {}", already_read);
            cnt += 1;
            if loop_read == 0 {
                if ring_buffer.all_write_ends_closed() || cnt > 3 {
                    return Ok(already_read);
                }
                drop(ring_buffer);
                yield_now();
                continue;
            }
            for _ in 0..loop_read {
                if let Some(byte_ref) = buf_iter.next() {
                    *byte_ref = ring_buffer.read_byte();
                    already_read += 1;
                    if already_read == want_to_read {
                        return Ok(want_to_read);
                    }
                } else {
                    return Ok(already_read);
                }
            }
        }
    }
}

impl Write for Pipe {
    fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        trace!("kernel: Pipe::write");
        assert!(self.writable());
        let want_to_write = buf.len();
        let mut buf_iter = buf.iter();
        let mut cnt = 0;
        let mut already_write = 0usize;
        loop {
            let mut ring_buffer = self.buffer.lock();
            let loop_write = ring_buffer.available_write();
            if loop_write == 0 {
                drop(ring_buffer);
                yield_now();
                continue;
            }
            cnt += 1;
            if Arc::strong_count(&self.buffer) < 2 || cnt > 3 {
                // 读入端关闭
                return Ok(already_write);
            }
            // write at most loop_write bytes
            for _ in 0..loop_write {
                if let Some(byte_ref) = buf_iter.next() {
                    // info!("write to pipe");
                    ring_buffer.write_byte(*byte_ref);
                    already_write += 1;
                    if already_write == want_to_write {
                        return Ok(want_to_write);
                    }
                } else {
                    return Ok(already_write);
                }
            }
        }
    }

    fn flush(&mut self) -> AxResult {
        Err(AxError::Unsupported) // 如果没有实现flush, 则返回Unsupported
    }
}

impl Seek for Pipe {
    fn seek(&mut self, _: axio::SeekFrom) -> AxResult<u64> {
        Err(AxError::Unsupported) // 如果没有实现seek, 则返回Unsupported
    }
}

impl FileExt for Pipe {
    fn executable(&self) -> bool {
        false
    }
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
}

impl FileIO for Pipe {
    fn get_type(&self) -> FileIOType {
        FileIOType::Pipe
    }

    fn is_hang_up(&self) -> bool {
        if self.readable {
            if self.buffer.lock().available_read() == 0
                && self.buffer.lock().all_write_ends_closed()
            {
                // 写入端关闭且缓冲区读完了
                true
            } else {
                false
            }
        } else {
            // 否则在写入端，只关心读入端是否被关闭
            Arc::strong_count(&self.buffer) < 2
        }
    }

    fn ready_to_read(&mut self) -> bool {
        self.readable && self.buffer.lock().available_read() != 0
    }

    fn ready_to_write(&mut self) -> bool {
        self.writable && self.buffer.lock().available_write() != 0
    }
}

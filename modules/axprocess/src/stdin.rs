use axerrno::{AxError, AxResult};
use axfs::monolithic_fs::{
    file_io::{FileExt, FileIO},
    FileIOType,
};
use axhal::console::{getchar, write_bytes};
use axio::{Read, Seek, SeekFrom, Write};
use axtask::yield_now;
/// stdin file for getting chars from console
pub struct Stdin;

/// stdout file for putting chars to console
pub struct Stdout;

/// stderr file for putting chars to console
pub struct Stderr;

impl Read for Stdin {
    fn read(&mut self, _buf: &mut [u8]) -> AxResult<usize> {
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
}

impl Write for Stdin {
    fn write(&mut self, _: &[u8]) -> AxResult<usize> {
        panic!("Cannot write to stdin!");
    }
    fn flush(&mut self) -> axio::Result {
        panic!("Flushing stdin")
    }
}

impl Seek for Stdin {
    fn seek(&mut self, _pos: SeekFrom) -> AxResult<u64> {
        Err(AxError::Unsupported) // 如果没有实现seek, 则返回Unsupported
    }
}

impl FileExt for Stdin {
    fn executable(&self) -> bool {
        false
    }
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
}

impl FileIO for Stdin {
    fn get_type(&self) -> FileIOType {
        FileIOType::Stdin
    }

    fn ready_to_read(&mut self) -> bool {
        true
    }

    fn ready_to_write(&mut self) -> bool {
        false
    }
}

impl Read for Stdout {
    fn read(&mut self, _: &mut [u8]) -> AxResult<usize> {
        panic!("Cannot read from stdin!");
    }
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        write_bytes(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> AxResult {
        // stdout is always flushed
        Ok(())
    }
}

impl Seek for Stdout {
    fn seek(&mut self, _pos: SeekFrom) -> AxResult<u64> {
        Err(AxError::Unsupported) // 如果没有实现seek, 则返回Unsupported
    }
}

impl FileExt for Stdout {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn executable(&self) -> bool {
        false
    }
}
impl FileIO for Stdout {
    fn get_type(&self) -> FileIOType {
        FileIOType::Stdout
    }

    fn ready_to_read(&mut self) -> bool {
        false
    }

    fn ready_to_write(&mut self) -> bool {
        true
    }
}

impl Read for Stderr {
    fn read(&mut self, _: &mut [u8]) -> AxResult<usize> {
        panic!("Cannot read from stdout!");
    }
}

impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        write_bytes(buf);
        Ok(buf.len())
    }

    /// Stderr is always flushed
    fn flush(&mut self) -> axio::Result {
        Ok(())
    }
}

impl Seek for Stderr {
    fn seek(&mut self, _pos: SeekFrom) -> AxResult<u64> {
        Err(AxError::Unsupported) // 如果没有实现seek, 则返回Unsupported
    }
}

impl FileExt for Stderr {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn executable(&self) -> bool {
        false
    }
}

impl FileIO for Stderr {
    fn get_type(&self) -> FileIOType {
        FileIOType::Stderr
    }

    fn ready_to_read(&mut self) -> bool {
        false
    }

    fn ready_to_write(&mut self) -> bool {
        true
    }
}

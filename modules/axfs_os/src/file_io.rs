use axerrno::{AxError, AxResult};

/// File I/O interface
pub trait FileIO: Send + Sync {
    /// Returns true if file is readable.
    fn readable(&self) -> bool;
    /// Returns true if file is writeable.
    fn writable(&self) -> bool;
    /// Reads data to the buffer.
    /// Returns the number of bytes read.
    fn read(&self, buf: &mut [u8]) -> AxResult<usize>;
    /// Writes data into the buffer.
    /// Returns the number of bytes written.
    fn write(&self, data: &[u8]) -> AxResult<usize>;
    /// Moves the cursor.
    /// Returns the new position.
    fn seek(&self, pos: usize) -> AxResult<u64>;
    /// Flushes buffer data.
    fn flush(&self) -> AxResult<()> {
        Err(AxError::Unsupported)
    }
}

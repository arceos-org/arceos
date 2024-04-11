use alloc::sync::Arc;
use axerrno::{AxError, AxResult};
use axfs::api::{FileIO, FileIOType, OpenFlags};
use axsync::Mutex;
use axtask::yield_now;
use bitflags::bitflags;

bitflags! {
    // https://sites.uclouvain.be/SystInfo/usr/include/sys/eventfd.h.html
    #[derive(Clone, Copy, Debug)]
    pub struct EventFdFlag: u32 {
        const EFD_SEMAPHORE = 0x1;
        const EFD_NONBLOCK = 0x800;
        const EFD_CLOEXEC = 0x80000;
    }
}

// https://man7.org/linux/man-pages/man2/eventfd2.2.html
pub struct EventFd {
    value: Arc<Mutex<u64>>,
    flags: u32,
}

impl EventFd {
    pub fn new(initval: u64, flags: u32) -> EventFd {
        EventFd {
            value: Arc::new(Mutex::new(initval)),
            flags,
        }
    }

    fn should_block(&self) -> bool {
        self.flags & EventFdFlag::EFD_NONBLOCK.bits() == 0
    }

    fn has_semaphore_set(&self) -> bool {
        self.flags & EventFdFlag::EFD_SEMAPHORE.bits() != 0
    }
}

impl FileIO for EventFd {
    fn read(&self, buf: &mut [u8]) -> AxResult<usize> {
        let len: usize = core::mem::size_of::<u64>();
        if buf.len() < len {
            return Err(AxError::InvalidInput);
        }

        loop {
            let mut value_guard = self.value.lock();
            // If EFD_SEMAPHORE was not specified and the eventfd counter has a nonzero value, then a read returns 8 bytes containing that value,
            // and the counter's value is reset to zero.
            if !self.has_semaphore_set() && *value_guard != 0 {
                buf[0..len].copy_from_slice(&value_guard.to_ne_bytes());
                *value_guard = 0;
                return Ok(len);
            }

            // If EFD_SEMAPHORE was specified and the eventfd counter has a nonzero value, then a read returns 8 bytes containing the value 1,
            // and the counter's value is decremented by 1.
            if self.has_semaphore_set() && *value_guard != 0 {
                let result: u64 = 1;
                buf[0..len].copy_from_slice(&result.to_ne_bytes());
                let _ = value_guard.checked_add_signed(-1);
                return Ok(len);
            }

            // If the eventfd counter is zero at the time of the call to read,
            // then the call either blocks until the counter becomes nonzero (at which time, the read proceeds as described above)
            // or fails with the error EAGAIN if the file descriptor has been made nonblocking.
            if *value_guard != 0 {
                buf[0..len].copy_from_slice(&value_guard.to_ne_bytes());
                return Ok(len);
            }

            if self.should_block() {
                drop(value_guard);
                yield_now()
            } else {
                return Err(AxError::WouldBlock);
            }
        }
    }

    fn write(&self, buf: &[u8]) -> AxResult<usize> {
        let len: usize = core::mem::size_of::<u64>();

        // A write fails with the error EINVAL if the size of the supplied buffer is less than 8 bytes,
        // or if an attempt is made to write the value 0xffffffffffffffff.
        let val = u64::from_ne_bytes(buf[0..len].try_into().unwrap());
        if buf.len() < 8 || val == u64::MAX {
            return Err(AxError::InvalidInput);
        }

        loop {
            let mut value_guard = self.value.lock();
            // The maximum value that may be stored in the counter is the largest unsigned 64-bit value minus 1 (i.e., 0xfffffffffffffffe).
            match value_guard.checked_add(val + 1) {
                // no overflow
                Some(_) => {
                    *value_guard += val;
                    return Ok(len);
                }
                // overflow
                None => {
                    if self.should_block() {
                        drop(value_guard);
                        yield_now()
                    } else {
                        return Err(AxError::WouldBlock);
                    }
                }
            }
        }
    }

    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        true
    }

    fn executable(&self) -> bool {
        false
    }

    fn get_type(&self) -> FileIOType {
        FileIOType::Other
    }

    // The file descriptor is readable if the counter has a value greater than 0
    fn ready_to_read(&self) -> bool {
        *self.value.lock() > 0
    }

    // The file descriptor is writable if it is possible to write a value of at least "1" without blocking.
    fn ready_to_write(&self) -> bool {
        *self.value.lock() < u64::MAX - 1
    }

    fn get_status(&self) -> OpenFlags {
        let mut status = OpenFlags::RDWR;
        if self.flags & EventFdFlag::EFD_NONBLOCK.bits() != 0 {
            status &= OpenFlags::NON_BLOCK;
        }
        if self.flags & EventFdFlag::EFD_CLOEXEC.bits() != 0 {
            status &= OpenFlags::CLOEXEC;
        }

        status
    }
}

#[cfg(test)]
mod tests {
    use super::EventFd;
    use axfs::api::FileIO;

    #[test]
    fn test_read() {
        let event_fd = EventFd::new(42, 0);
        let event_fd_val = 0u64;
        let len = event_fd.read(&mut event_fd_val.to_ne_bytes()).unwrap();

        assert_eq!(42, event_fd_val);
        assert_eq!(4, len);
    }

    #[test]
    fn test_read_with_bad_input() {
        let event_fd = EventFd::new(42, 0);
        let event_fd_val = 0u32;
        let result = event_fd.read(&mut event_fd_val.to_ne_bytes());
        assert_eq!(Err(AxError::InvalidInput), result);
    }

    #[test]
    fn test_write() {
        let event_fd = EventFd::new(42, 0);
        let val = 12u64;
        event_fd.write(&val.to_ne_bytes()[0..core::mem::size_of::<u64>()]);

        let event_fd_val = 0u64;
        event_fd.read(&mut event_fd_val.to_ne_bytes()).unwrap();
        assert_eq!(54, event_fd_val);
    }
}

use bitflags::bitflags;

bitflags! {
    /// https://sites.uclouvain.be/SystInfo/usr/include/sys/eventfd.h.html
    #[derive(Clone, Copy, Debug)]
    pub struct EventFdFlag: u32 {
        const EFD_SEMAPHORE = 0x1;
        const EFD_NONBLOCK  = 0x800;
        const EFD_CLOEXEC   = 0x80000;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventFdWriteResult {
    OK,
    /// an attempt is made to write the value 0xffffffffffffffff (f64::MAX)
    BadInput,
    NotReady,
}

/// https://man7.org/linux/man-pages/man2/eventfd2.2.html
pub struct EventFd {
    value: u64,
    flags: u32,
}

impl EventFd {
    pub fn new(initval: u64, flags: u32) -> EventFd {
        EventFd {
            value: initval,
            flags,
        }
    }

    pub fn read(&mut self) -> Option<u64> {
        // If EFD_SEMAPHORE was not specified and the eventfd counter has a nonzero value, then a read returns 8 bytes containing that value,
        // and the counter's value is reset to zero.
        if !self.is_semaphore_set() && self.value != 0 {
            let result = self.value;
            self.value = 0;
            return Some(result);
        }

        // If EFD_SEMAPHORE was specified and the eventfd counter has a nonzero value, then a read returns 8 bytes containing the value 1,
        // and the counter's value is decremented by 1.
        if self.is_semaphore_set() && self.value != 0 {
            self.value -= 1;
            return Some(1u64);
        }

        // If the eventfd counter is zero at the time of the call to read,
        // then the call either blocks until the counter becomes nonzero (at which time, the read proceeds as described above)
        // or fails with the error EAGAIN if the file descriptor has been made nonblocking.
        None
    }

    pub fn write(&mut self, val: u64) -> EventFdWriteResult {
        if val == u64::MAX {
            return EventFdWriteResult::BadInput;
        }

        match self.value.checked_add(val + 1) {
            // no overflow
            Some(_) => {
                self.value += val;
                EventFdWriteResult::OK
            }
            // overflow
            None => EventFdWriteResult::NotReady,
        }
    }

    pub fn is_flag_set(&self, flag: EventFdFlag) -> bool {
        self.flags & flag.bits() != 0
    }

    fn is_semaphore_set(&self) -> bool {
        self.is_flag_set(EventFdFlag::EFD_SEMAPHORE)
    }
}

/// create eventfd with flags of zero default value
pub fn create_eventfd(value: u64) -> EventFd {
    value.into()
}

pub fn create_eventfd_with_flags(value: u64, flags: u32) -> EventFd {
    (value, flags).into()
}

impl From<u64> for EventFd {
    fn from(value: u64) -> Self {
        EventFd {
            value: value,
            flags: 0,
        }
    }
}

impl From<(u64, u32)> for EventFd {
    fn from((value, flags): (u64, u32)) -> Self {
        EventFd { value, flags }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_flags_should_not_set() {
        let event_fd: EventFd = 42.into();
        assert!(!event_fd.is_flag_set(EventFdFlag::EFD_SEMAPHORE));
        assert!(!event_fd.is_flag_set(EventFdFlag::EFD_NONBLOCK));
        assert!(!event_fd.is_flag_set(EventFdFlag::EFD_CLOEXEC));
    }

    #[test]
    fn only_efd_semaphore_is_set() {
        let event_fd: EventFd = (42, 0x1).into();
        assert!(event_fd.is_flag_set(EventFdFlag::EFD_SEMAPHORE));
        assert!(!event_fd.is_flag_set(EventFdFlag::EFD_NONBLOCK));
        assert!(!event_fd.is_flag_set(EventFdFlag::EFD_CLOEXEC));
    }

    #[test]
    fn only_efd_nonblock_is_set() {
        let event_fd: EventFd = (42, 0x800).into();
        assert!(!event_fd.is_flag_set(EventFdFlag::EFD_SEMAPHORE));
        assert!(event_fd.is_flag_set(EventFdFlag::EFD_NONBLOCK));
        assert!(!event_fd.is_flag_set(EventFdFlag::EFD_CLOEXEC));
    }

    #[test]
    fn only_efd_cloexec_is_set() {
        let event_fd: EventFd = (42, 0x80000).into();
        assert!(!event_fd.is_flag_set(EventFdFlag::EFD_SEMAPHORE));
        assert!(!event_fd.is_flag_set(EventFdFlag::EFD_NONBLOCK));
        assert!(event_fd.is_flag_set(EventFdFlag::EFD_CLOEXEC));
    }

    #[test]
    fn read_with_efd_semaphore_not_set() {
        let mut event_fd: EventFd = 42.into();
        assert_eq!(Some(42), event_fd.read());
        assert_eq!(None, event_fd.read());
    }

    #[test]
    fn read_with_efd_semaphore_set() {
        let mut event_fd: EventFd = create_eventfd_with_flags(2, EventFdFlag::EFD_SEMAPHORE.bits());
        assert_eq!(Some(1), event_fd.read());
        assert_eq!(Some(1), event_fd.read());
        assert_eq!(None, event_fd.read());
    }

    #[test]
    fn write_max_value() {
        let mut event_fd: EventFd = 42.into();
        assert_eq!(EventFdWriteResult::BadInput, event_fd.write(u64::MAX))
    }

    #[test]
    fn test_overflow_write() {
        let mut event_fd: EventFd = (u64::MAX - 1).into();
        assert_eq!(EventFdWriteResult::NotReady, event_fd.write(2))
    }

    #[test]
    fn test_non_overflow_write() {
        let mut event_fd: EventFd = (u64::MAX - 2).into();
        assert_eq!(EventFdWriteResult::OK, event_fd.write(1));

        assert_eq!(Some(u64::MAX - 1), event_fd.read());
    }
}

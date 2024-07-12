//! System Real Time Clock (RTC) Drivers for riscv based on goldfish.
//! Ref: <https://github.com/torvalds/linux/blob/master/drivers/rtc/rtc-goldfish.c>

#![cfg_attr(not(test), no_std)]

const RTC_TIME_LOW: usize = 0x00;
const RTC_TIME_HIGH: usize = 0x04;

const NSEC_PER_SEC: u64 = 1_000_000_000;

/// The System Real Time Clock structure for riscv based on goldfish.
pub struct Rtc {
    base_address: usize,
}

impl Rtc {
    unsafe fn read(&self, reg: usize) -> u32 {
        core::ptr::read_volatile((self.base_address + reg) as *const u32)
    }

    unsafe fn write(&self, reg: usize, value: u32) {
        core::ptr::write_volatile((self.base_address + reg) as *mut u32, value);
    }
}

impl Rtc {
    /// Construct a new goldfish RTC structure.
    ///
    /// `base_addr` represents the device address
    ///  (which can be obtained from the device tree).
    pub fn new(base_address: usize) -> Self {
        Rtc { base_address }
    }

    /// Returns the current time in seconds since UNIX epoch.
    pub fn get_unix_timestamp(&self) -> u64 {
        let low = unsafe { self.read(RTC_TIME_LOW) as u64 };
        let high = unsafe { self.read(RTC_TIME_HIGH) as u64 };
        ((high << 32) | low) / NSEC_PER_SEC
    }

    /// Sets the current time in seconds since UNIX epoch.
    pub fn set_unix_timestamp(&self, unix_time: u64) {
        let time_nanos = unix_time * NSEC_PER_SEC;
        unsafe {
            self.write(RTC_TIME_HIGH, (time_nanos >> 32) as u32);
            self.write(RTC_TIME_LOW, time_nanos as u32);
        }
    }
}

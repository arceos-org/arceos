//! System Real Time Clock (RTC) Drivers for aarch64 based on PL031.

#![cfg_attr(not(test), no_std)]

const RTC_DR: usize = 0x00; //Data Register
const RTC_LR: usize = 0x08; //Load Register

/// The System Real Time Clock structure for aarch64 based on PL031.
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
    /// Construct a new PL031 RTC structure.
    ///
    /// `base_addr` represents the device address
    ///  (which can be obtained from the device tree).
    pub fn new(base_address: usize) -> Self {
        Rtc { base_address }
    }

    /// Returns the current time in seconds since UNIX epoch.
    pub fn get_unix_timestamp(&self) -> u64 {
        unsafe { self.read(RTC_DR) as u64 }
    }

    /// Sets the current time in seconds since UNIX epoch.
    pub fn set_unix_timestamp(&self, unix_time: u64) {
        unsafe { self.write(RTC_LR, unix_time as u32) }
    }
}

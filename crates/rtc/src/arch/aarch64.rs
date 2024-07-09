//! System Real Time Clock (RTC) Drivers for aarch64 based on PL031.

const RTC_DR: usize = 0x00;
const RTC_MR: usize = 0x04;
const RTC_LR: usize = 0x08;
const RTC_CR: usize = 0x0c;
/// Interrupt mask and set register
const RTC_IRQ_MASK: usize = 0x10;
/// Raw interrupt status
const RTC_RAW_IRQ_STATUS: usize = 0x14;
/// Masked interrupt status
const RTC_MASK_IRQ_STATUS: usize = 0x18;
/// Interrupt clear register
const RTC_IRQ_CLEAR: usize = 0x1c;

pub struct Rtc {
    base_address: usize,
}

impl Rtc {
    pub fn new(base_address: usize) -> Self {
        Rtc { base_address }
    }

    pub fn get_unix_timestamp(&self) -> u64 {
        unsafe { ((self.base_address + RTC_DR) as *mut u32).read() as u64 }
    }
}

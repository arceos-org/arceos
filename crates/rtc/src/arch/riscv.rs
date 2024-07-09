//! System Real Time Clock (RTC) Drivers for riscv based on goldfish.

// rtc@101000 {
//     interrupts = <0x0b>;
//     interrupt-parent = <0x03>;
//     reg = <0x00 0x101000 0x00 0x1000>;
//     compatible = "google,goldfish-rtc";
// };

const RTC_TIME_LOW: usize = 0x00;
const RTC_TIME_HIGH: usize = 0x04;

pub struct Rtc {
    base_address: usize,
}

impl Rtc {
    pub fn new(base_address: usize) -> Self {
        Rtc { base_address }
    }

    pub fn get_unix_timestamp(&self) -> u64 {
        const NSEC_PER_SEC: u64 = 1000_000_000;

        let low = unsafe { ((self.base_address + RTC_TIME_LOW) as *mut u32).read() as u64 };
        let high = unsafe { ((self.base_address + RTC_TIME_HIGH) as *mut u32).read() as u64 };
        ((high << 32) | low) / NSEC_PER_SEC
    }
}

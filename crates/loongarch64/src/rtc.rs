use alloc::format;
use alloc::string::{String, ToString};
use core::fmt::{Debug, Formatter};
use rlibc::memcmp;
use crate::loongarch64::{ls7a_read_w, ls7a_write_w, LS7A_RTC_REG_BASE};
use crate::println;
use bit_field::BitField;

pub const RTC_YEAR: usize = 0x30;
pub const RTC_TOYREAD0: usize = 0x2c; //月日时分
pub const RTC_CTRL: usize = 0x40;

pub struct RtcTime {
    pub year: u32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl Debug for RtcTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}-{}-{} {}:{}:{}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

impl ToString for RtcTime {
    fn to_string(&self) -> String {
        format!(
            "{}:{}:{}\n{}-{}-{}",
            self.hour, self.minute, self.second, self.year, self.month, self.day
        )
    }
}

/// 编译器会报出找不到bcmp的错误，这里将其实现为memcmp
#[no_mangle]
pub unsafe extern "C" fn bcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    memcmp(s1, s2, n)
}


pub fn rtc_time_read() -> RtcTime {
    let value = ls7a_read_w(LS7A_RTC_REG_BASE + RTC_TOYREAD0);
    let sec = (value >> 4) & 0x3f;
    let min = (value >> 10) & 0x3f;
    let mut hour = (value >> 16) & 0x1f;
    let day = (value >> 21) & 0x1f;
    let mon = (value >> 26) & 0x3f;
    let year = ls7a_read_w(LS7A_RTC_REG_BASE + RTC_YEAR) + 1900;
    hour = (hour + 8) % 24;
    return RtcTime {
        year,
        month: mon,
        day,
        hour,
        minute: min,
        second: sec,
    };
}
pub fn check_rtc() {
    let val = ls7a_read_w(LS7A_RTC_REG_BASE + RTC_CTRL);
    println!(
        "RTC enable:{}, TOY enable:{}",
        val.get_bit(13),
        val.get_bit(11)
    );
}

pub fn rtc_init() {
    let mut val = ls7a_read_w(LS7A_RTC_REG_BASE + RTC_CTRL);
    val.set_bit(13, true);
    val.set_bit(11, true);
    ls7a_write_w(LS7A_RTC_REG_BASE + RTC_CTRL, val);
}

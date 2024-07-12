//! System Real Time Clock (RTC) Drivers for x86_64 based on CMOS.
//! Ref:
//!  * <https://wiki.osdev.org/RTC>
//!  * <https://wiki.osdev.org/CMOS#The_Real-Time_Clock>
//!  * <https://github.com/hermit-os/kernel/blob/main/src/arch/x86_64/kernel/systemtime.rs>
//!  * <https://github.com/syswonder/ruxos/blob/main/modules/ruxhal/src/platform/x86_pc/rtc.rs>

#![cfg_attr(not(test), no_std)]

const CMOS_SECOND_REGISTER: u8 = 0x00;
const CMOS_MINUTE_REGISTER: u8 = 0x02;
const CMOS_HOUR_REGISTER: u8 = 0x04;
const CMOS_DAY_REGISTER: u8 = 0x07;
const CMOS_MONTH_REGISTER: u8 = 0x08;
const CMOS_YEAR_REGISTER: u8 = 0x09;
const CMOS_STATUS_REGISTER_A: u8 = 0x0A;
const CMOS_STATUS_REGISTER_B: u8 = 0x0B;

const CMOS_UPDATE_IN_PROGRESS_FLAG: u8 = 1 << 7;
const CMOS_24_HOUR_FORMAT_FLAG: u8 = 1 << 1;
const CMOS_BINARY_FORMAT_FLAG: u8 = 1 << 2;
const CMOS_12_HOUR_PM_FLAG: u8 = 0x80;

/// The System Real Time Clock structure for x86 based on CMOS.
pub struct Rtc {
    cmos_format: u8,
}

impl Rtc {
    const fn is_24_hour_format(&self) -> bool {
        self.cmos_format & CMOS_24_HOUR_FORMAT_FLAG > 0
    }

    const fn is_binary_format(&self) -> bool {
        self.cmos_format & CMOS_BINARY_FORMAT_FLAG > 0
    }

    fn read_datetime_register(&self, register: u8) -> u8 {
        let value = read_cmos_register(register);

        // Every date/time register may either be in binary or in BCD format.
        // Convert BCD values if necessary.
        if self.is_binary_format() {
            value
        } else {
            convert_bcd_value(value)
        }
    }

    fn read_all_values(&self) -> u64 {
        // Reading year, month, and day is straightforward.
        let year = u16::from(self.read_datetime_register(CMOS_YEAR_REGISTER)) + 2000;
        let month = self.read_datetime_register(CMOS_MONTH_REGISTER);
        let day = self.read_datetime_register(CMOS_DAY_REGISTER);

        // The hour register is a bitch.
        // On top of being in either binary or BCD format, it may also be in 12-hour
        // or 24-hour format.
        let mut hour = read_cmos_register(CMOS_HOUR_REGISTER);
        let mut is_pm = false;

        // Check and mask off a potential PM flag if the hour is given in 12-hour format.
        if !self.is_24_hour_format() {
            is_pm = time_is_pm(hour);
            hour &= !CMOS_12_HOUR_PM_FLAG;
        }

        // Now convert a BCD number to binary if necessary (after potentially masking off the PM flag above).
        if !self.is_binary_format() {
            hour = convert_bcd_value(hour);
        }

        // If the hour is given in 12-hour format, do the necessary calculations to convert it into 24 hours.
        if !self.is_24_hour_format() {
            if hour == 12 {
                // 12:00 AM is 00:00 and 12:00 PM is 12:00 (see is_pm below) in 24-hour format.
                hour = 0;
            }

            if is_pm {
                // {01:00 PM, 02:00 PM, ...} is {13:00, 14:00, ...} in 24-hour format.
                hour += 12;
            }
        }

        // The minute and second registers are straightforward again.
        let minute = self.read_datetime_register(CMOS_MINUTE_REGISTER);
        let second = self.read_datetime_register(CMOS_SECOND_REGISTER);

        // Convert it all to seconds and return the result.
        seconds_from_date(year, month, day, hour, minute, second)
    }
}

impl Rtc {
    /// Construct a new CMOS RTC structure.
    pub fn new() -> Self {
        Self {
            cmos_format: read_cmos_register(CMOS_STATUS_REGISTER_B),
        }
    }

    /// Returns the current time in seconds since UNIX epoch.
    ///
    /// Note: The call to this RTC method requires the interrupt to be disabled, otherwise the value read may be inaccurate.
    pub fn get_unix_timestamp(&self) -> u64 {
        loop {
            // If a clock update is currently in progress, wait until it is finished.
            while read_cmos_register(CMOS_STATUS_REGISTER_A) & CMOS_UPDATE_IN_PROGRESS_FLAG > 0 {
                core::hint::spin_loop();
            }

            // Get the current time in seconds since the epoch.
            let seconds_since_epoch_1 = self.read_all_values();

            // If the clock is already updating the time again, the read values may be inconsistent
            // and we have to repeat this process.
            if read_cmos_register(CMOS_STATUS_REGISTER_A) & CMOS_UPDATE_IN_PROGRESS_FLAG > 0 {
                continue;
            }

            // Get the current time again and verify that it's the same we last read.
            let seconds_since_epoch_2 = self.read_all_values();
            if seconds_since_epoch_1 == seconds_since_epoch_2 {
                // Both times are identical, so we have read consistent values and can exit the loop.
                return seconds_since_epoch_1;
            }
        }
    }

    /// Sets the current time in seconds since UNIX epoch.
    pub fn set_unix_timestamp(&self, unix_time: u64) {
        let secs = unix_time as u32;

        // Calculate date and time
        let t = secs;
        let mut tdiv = t / 86400;
        let mut tt = t % 86400;
        let mut hour = tt / 3600;
        tt %= 3600;
        let mut min = tt / 60;
        tt %= 60;
        let mut sec = tt;
        let mut year = 1970;
        let mut mon = 1;

        while tdiv >= 365 {
            let days = if is_leap_year(year) { 366 } else { 365 };
            if tdiv >= days {
                tdiv -= days;
                year += 1;
            } else {
                break;
            }
        }

        while tdiv > 0 {
            let days = days_in_month(mon, year);
            if u64::from(tdiv) >= days {
                tdiv -= days as u32;
                mon += 1;
            } else {
                break;
            }
        }

        let mut mday = tdiv + 1;

        year -= 2000;

        if !self.is_binary_format() {
            sec = convert_binary_value(sec as _) as u32;
            min = convert_binary_value(min as _) as u32;
            mday = convert_binary_value(mday as _) as u32;
            mon = convert_binary_value(mon as _) as u64;
            year = convert_binary_value(year as _) as u64;
        }

        let mut bcd_value = hour % 10;
        let tens = hour / 10;
        if hour >= 12 {
            bcd_value |= CMOS_12_HOUR_PM_FLAG as u32;
        }
        bcd_value |= tens << 4;
        hour = bcd_value;

        write_cmos_register(CMOS_SECOND_REGISTER, sec as u8);
        write_cmos_register(CMOS_MINUTE_REGISTER, min as u8);
        write_cmos_register(CMOS_HOUR_REGISTER, hour as u8);
        write_cmos_register(CMOS_DAY_REGISTER, mday as u8);
        write_cmos_register(CMOS_MONTH_REGISTER, mon as u8);
        write_cmos_register(CMOS_YEAR_REGISTER, year as u8);
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        use x86_64::instructions::port::Port;
        const CMOS_COMMAND_PORT: u16 = 0x70;
        const CMOS_DATA_PORT: u16 = 0x71;
        const CMOS_DISABLE_NMI: u8 = 1 << 7;

        static mut COMMAND_PORT: Port<u8> = Port::new(CMOS_COMMAND_PORT);
        static mut DATA_PORT: Port<u8> = Port::new(CMOS_DATA_PORT);

        fn read_cmos_register(register: u8) -> u8 {
            unsafe {
                COMMAND_PORT.write(CMOS_DISABLE_NMI | register);
                DATA_PORT.read()
            }
        }

        fn write_cmos_register(register: u8, value: u8) {
            unsafe {
                COMMAND_PORT.write(CMOS_DISABLE_NMI | register);
                DATA_PORT.write(value)
            }
        }

    } else {
        fn read_cmos_register(_register: u8) -> u8 {
            0
        }
        fn write_cmos_register(_register: u8, _value: u8) {}
    }
}

const fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

const fn days_in_month(month: u64, year: u64) -> u64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

const fn time_is_pm(hour: u8) -> bool {
    hour & CMOS_12_HOUR_PM_FLAG > 0
}

/// Returns the binary value for a given value in BCD (Binary-Coded Decimal).
/// Refer to <https://wiki.osdev.org/CMOS#Format_of_Bytes>.
const fn convert_bcd_value(bcd: u8) -> u8 {
    ((bcd & 0xF0) >> 1) + ((bcd & 0xF0) >> 3) + (bcd & 0xf)
}

/// Returns the BCD (Binary-Coded Decimal) value for a given value in binary.
const fn convert_binary_value(binary: u8) -> u8 {
    ((binary / 10) << 4) | (binary % 10)
}

/// Returns the number of seconds since the epoch from a given date.
/// Inspired by Linux Kernel's mktime64(), see kernel/time/time.c.
fn seconds_from_date(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> u64 {
    let (m, y) = if month > 2 {
        (u64::from(month - 2), u64::from(year))
    } else {
        (u64::from(month + 12 - 2), u64::from(year - 1))
    };

    let days_since_epoch =
        (y / 4 - y / 100 + y / 400 + 367 * m / 12 + u64::from(day)) + y * 365 - 719_499;
    let hours_since_epoch = days_since_epoch * 24 + u64::from(hour);
    let minutes_since_epoch = hours_since_epoch * 60 + u64::from(minute);
    minutes_since_epoch * 60 + u64::from(second)
}

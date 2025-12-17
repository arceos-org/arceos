use core::ffi::c_int;

use crate::ctypes;

const MONTH_DAYS: [[c_int; 12]; 2] = [
    // Non-leap years:
    [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
    // Leap years:
    [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
];

#[inline(always)]
fn leap_year(year: c_int) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Normalize a time value to be within a given range [0, modulus).
/// Returns the normalized value and the number of carry-overs.
#[inline]
fn normalize(value: i64, modulus: i64) -> (c_int, i64) {
    let mut val = value;
    let mut carry = 0i64;
    
    if val < 0 {
        carry = (val - modulus + 1) / modulus;
        val -= carry * modulus;
    }
    if val >= modulus {
        carry = val / modulus;
        val -= carry * modulus;
    }
    
    (val as c_int, carry)
}

/// Count the number of leap years from year1 (inclusive) to year2 (exclusive).
/// Handles negative ranges correctly.
#[inline]
fn count_leap_years(year1: i64, year2: i64) -> i64 {
    if year1 >= year2 {
        return 0;
    }
    
    let y1 = year1 - 1;
    let y2 = year2 - 1;
    
    let leap1 = y1 / 4 - y1 / 100 + y1 / 400;
    let leap2 = y2 / 4 - y2 / 100 + y2 / 400;
    
    leap2 - leap1
}

/// Calculate the day of the week (0 = Sunday, 6 = Saturday).
/// Uses Zeller's congruence algorithm.
#[inline]
fn calc_wday(year: c_int, month: c_int, day: c_int) -> c_int {
    let mut y = year as i64;
    let mut m = month as i64 + 1; // month is 0-11, we need 1-12
    
    // January and February are counted as months 13 and 14 of the previous year
    if m < 3 {
        m += 12;
        y -= 1;
    }
    
    let c = y / 100;
    let y = y % 100;
    
    let w = (day as i64 + (13 * (m + 1)) / 5 + y + y / 4 + c / 4 - 2 * c) % 7;
    
    // Convert to Sunday = 0
    ((w + 6) % 7) as c_int
}

/// Calculate the day of the year (0-365).
#[inline]
fn calc_yday(year: c_int, month: c_int, day: c_int) -> c_int {
    let leap = if leap_year(year) { 1 } else { 0 };
    let mut yday = day - 1;
    
    for i in 0..month {
        yday += MONTH_DAYS[leap][i as usize];
    }
    
    yday
}

/// Convert broken-down time into time since the Epoch.
/// 
/// This implementation:
/// 1. Normalizes all input fields to their valid ranges
/// 2. Updates tm_wday and tm_yday based on the normalized date
/// 3. Uses O(1) algorithm to compute days since epoch
/// 4. Handles tm_isdst (currently assumes no DST adjustment)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mktime(t: *mut ctypes::tm) -> ctypes::time_t {
    // Step 1: Normalize all fields
    
    // Normalize seconds (0-59)
    let (sec, min_carry) = normalize(unsafe { (*t).tm_sec as i64 }, 60);
    
    // Normalize minutes (0-59)
    let (min, hour_carry) = normalize(unsafe { (*t).tm_min as i64 } + min_carry, 60);
    
    // Normalize hours (0-23)
    let (hour, day_carry) = normalize(unsafe { (*t).tm_hour as i64 } + hour_carry, 24);
    
    // Normalize months (0-11)
    let (month, year_carry) = normalize(unsafe { (*t).tm_mon as i64 }, 12);
    
    // Calculate year after month normalization
    let mut year = (unsafe { (*t).tm_year as i64 } + 1900 + year_carry) as c_int;
    let mut month = month;
    
    // Normalize days (handle day_carry and ensure day is in valid range for the month)
    let mut day = unsafe { (*t).tm_mday } + day_carry as c_int;
    
    // Adjust for days outside the valid range of the month
    while day < 1 {
        month -= 1;
        if month < 0 {
            month = 11;
            year -= 1;
        }
        let leap = if leap_year(year) { 1 } else { 0 };
        day += MONTH_DAYS[leap][month as usize];
    }
    
    loop {
        let leap = if leap_year(year) { 1 } else { 0 };
        let days_in_month = MONTH_DAYS[leap][month as usize];
        
        if day <= days_in_month {
            break;
        }
        
        day -= days_in_month;
        month += 1;
        if month > 11 {
            month = 0;
            year += 1;
        }
    }
    
    // Step 2: Calculate day of week and day of year
    let wday = calc_wday(year, month, day);
    let yday = calc_yday(year, month, day);
    
    // Step 3: Calculate time_t using O(1) algorithm
    // Number of days since epoch (Jan 1, 1970)
    let year_diff = year as i64 - 1970;
    
    // Calculate days from years
    let mut days = year_diff * 365;
    
    // Add leap days
    if year_diff >= 0 {
        days += count_leap_years(1970, year as i64);
    } else {
        days -= count_leap_years(year as i64, 1970);
    }
    
    // Add days from months
    let leap = if leap_year(year) { 1 } else { 0 };
    for i in 0..month {
        days += MONTH_DAYS[leap][i as usize] as i64;
    }
    
    // Add days
    days += (day - 1) as i64;
    
    // Calculate total seconds
    let result = days * 86400 + hour as i64 * 3600 + min as i64 * 60 + sec as i64;
    
    // Step 4: Update the tm structure with normalized values
    unsafe {
        (*t).tm_sec = sec;
        (*t).tm_min = min;
        (*t).tm_hour = hour;
        (*t).tm_mday = day;
        (*t).tm_mon = month;
        (*t).tm_year = year - 1900;
        (*t).tm_wday = wday;
        (*t).tm_yday = yday;
        
        // For tm_isdst: A negative value causes mktime to attempt to determine DST status.
        // Since we don't have timezone information, we set it to -1 to indicate unknown.
        // A proper implementation would need timezone database support.
        if (*t).tm_isdst < 0 {
            (*t).tm_isdst = -1; // Unknown
        }
    }
    
    result as ctypes::time_t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(C)]
    struct TestTm {
        tm_sec: c_int,
        tm_min: c_int,
        tm_hour: c_int,
        tm_mday: c_int,
        tm_mon: c_int,
        tm_year: c_int,
        tm_wday: c_int,
        tm_yday: c_int,
        tm_isdst: c_int,
    }

    #[test]
    fn test_epoch() {
        let mut t = TestTm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 1,
            tm_mon: 0,
            tm_year: 70, // 1970
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: -1,
        };

        let result = unsafe { mktime(&mut t as *mut TestTm as *mut ctypes::tm) };
        assert_eq!(result, 0, "Epoch should be 0");
        assert_eq!(t.tm_wday, 4, "Jan 1, 1970 is Thursday (4)");
        assert_eq!(t.tm_yday, 0, "Jan 1 is day 0 of the year");
    }

    #[test]
    fn test_leap_year_date() {
        let mut t = TestTm {
            tm_sec: 30,
            tm_min: 15,
            tm_hour: 6,
            tm_mday: 29,
            tm_mon: 1, // February
            tm_year: 120, // 2020
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: -1,
        };

        let result = unsafe { mktime(&mut t as *mut TestTm as *mut ctypes::tm) };
        // Expected: 1582956930
        assert_eq!(result, 1582956930, "Feb 29, 2020 06:15:30");
        assert_eq!(t.tm_wday, 6, "Feb 29, 2020 is Saturday (6)");
        assert_eq!(t.tm_yday, 59, "Feb 29, 2020 is day 59 of the year");
    }

    #[test]
    fn test_normalization_seconds() {
        let mut t = TestTm {
            tm_sec: 60, // Should normalize to next minute
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 1,
            tm_mon: 0,
            tm_year: 124, // 2024
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: -1,
        };

        let result = unsafe { mktime(&mut t as *mut TestTm as *mut ctypes::tm) };
        assert_eq!(t.tm_sec, 0, "Seconds should normalize to 0");
        assert_eq!(t.tm_min, 1, "Minutes should be incremented");
        // Expected result for 2024-01-01 00:01:00
        assert_eq!(result, 1704067260);
    }

    #[test]
    fn test_normalization_month_overflow() {
        let mut t = TestTm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 12,
            tm_mday: 15,
            tm_mon: 12, // Should normalize to January of next year
            tm_year: 124, // 2024 -> 2025
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: -1,
        };

        let result = unsafe { mktime(&mut t as *mut TestTm as *mut ctypes::tm) };
        assert_eq!(t.tm_mon, 0, "Month should normalize to 0 (January)");
        assert_eq!(t.tm_year, 125, "Year should be 125 (2025)");
        assert_eq!(t.tm_mday, 15, "Day should remain 15");
        // Expected result for 2025-01-15 12:00:00
        assert_eq!(result, 1736942400);
    }

    #[test]
    fn test_normalization_month_underflow() {
        let mut t = TestTm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 12,
            tm_mday: 15,
            tm_mon: -1, // Should normalize to December of previous year
            tm_year: 124, // 2024 -> 2023
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: -1,
        };

        let result = unsafe { mktime(&mut t as *mut TestTm as *mut ctypes::tm) };
        assert_eq!(t.tm_mon, 11, "Month should normalize to 11 (December)");
        assert_eq!(t.tm_year, 123, "Year should be 123 (2023)");
        assert_eq!(t.tm_mday, 15, "Day should remain 15");
        // Expected result for 2023-12-15 12:00:00
        assert_eq!(result, 1702641600);
    }

    #[test]
    fn test_normalization_day_overflow() {
        let mut t = TestTm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 32, // Should normalize to February 1
            tm_mon: 0, // January
            tm_year: 124, // 2024
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: -1,
        };

        let result = unsafe { mktime(&mut t as *mut TestTm as *mut ctypes::tm) };
        assert_eq!(t.tm_mday, 1, "Day should normalize to 1");
        assert_eq!(t.tm_mon, 1, "Month should be 1 (February)");
        assert_eq!(t.tm_year, 124, "Year should remain 124 (2024)");
        // Expected result for 2024-02-01 00:00:00
        assert_eq!(result, 1706745600);
    }

    #[test]
    fn test_normalization_day_underflow() {
        let mut t = TestTm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0, // Should normalize to December 31 of previous year
            tm_mon: 0, // January
            tm_year: 124, // 2024 -> 2023
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: -1,
        };

        let result = unsafe { mktime(&mut t as *mut TestTm as *mut ctypes::tm) };
        assert_eq!(t.tm_mday, 31, "Day should normalize to 31");
        assert_eq!(t.tm_mon, 11, "Month should be 11 (December)");
        assert_eq!(t.tm_year, 123, "Year should be 123 (2023)");
        // Expected result for 2023-12-31 00:00:00
        assert_eq!(result, 1703980800);
    }

    #[test]
    fn test_large_year_performance() {
        // This test ensures O(1) time complexity - should complete instantly
        let mut t = TestTm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 1,
            tm_mon: 0,
            tm_year: 8100, // Year 10000
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: -1,
        };

        let result = unsafe { mktime(&mut t as *mut TestTm as *mut ctypes::tm) };
        // Expected result for year 10000-01-01
        assert_eq!(result, 253402300800);
        assert_eq!(t.tm_wday, 6, "Jan 1, 10000 is Saturday (6)");
    }

    #[test]
    fn test_pre_epoch() {
        let mut t = TestTm {
            tm_sec: 59,
            tm_min: 59,
            tm_hour: 23,
            tm_mday: 31,
            tm_mon: 11, // December
            tm_year: 69, // 1969
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: -1,
        };

        let result = unsafe { mktime(&mut t as *mut TestTm as *mut ctypes::tm) };
        assert_eq!(result, -1, "Dec 31, 1969 23:59:59 should be -1");
        assert_eq!(t.tm_wday, 3, "Dec 31, 1969 is Wednesday (3)");
        assert_eq!(t.tm_yday, 364, "Dec 31 is day 364 of the year");
    }

    #[test]
    fn test_helper_functions() {
        // Test leap_year
        assert!(leap_year(2000), "2000 is a leap year");
        assert!(leap_year(2004), "2004 is a leap year");
        assert!(!leap_year(1900), "1900 is not a leap year");
        assert!(!leap_year(2001), "2001 is not a leap year");

        // Test normalize
        let (val, carry) = normalize(0, 60);
        assert_eq!(val, 0);
        assert_eq!(carry, 0);

        let (val, carry) = normalize(60, 60);
        assert_eq!(val, 0);
        assert_eq!(carry, 1);

        let (val, carry) = normalize(-1, 60);
        assert_eq!(val, 59);
        assert_eq!(carry, -1);

        // Test count_leap_years
        assert_eq!(count_leap_years(1970, 2000), 7); // 1972, 1976, ..., 1996
        assert_eq!(count_leap_years(2000, 2001), 1); // 2000
    }
}

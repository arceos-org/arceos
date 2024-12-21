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

/// Convert broken-down time into time since the Epoch.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mktime(t: *mut ctypes::tm) -> ctypes::time_t {
    let mut year = (*t).tm_year + 1900;
    let mut month = (*t).tm_mon;
    let mut day = (*t).tm_mday as i64 - 1;

    let leap = if leap_year(year) { 1 } else { 0 };

    if year < 1970 {
        day = MONTH_DAYS[if leap_year(year) { 1 } else { 0 }][(*t).tm_mon as usize] as i64 - day;

        while year < 1969 {
            year += 1;
            day += if leap_year(year) { 366 } else { 365 };
        }

        while month < 11 {
            month += 1;
            day += MONTH_DAYS[leap][month as usize] as i64;
        }

        (-(day * (60 * 60 * 24)
            - (((*t).tm_hour as i64) * (60 * 60) + ((*t).tm_min as i64) * 60 + (*t).tm_sec as i64)))
            as ctypes::time_t
    } else {
        while year > 1970 {
            year -= 1;
            day += if leap_year(year) { 366 } else { 365 };
        }

        while month > 0 {
            month -= 1;
            day += MONTH_DAYS[leap][month as usize] as i64;
        }

        (day * (60 * 60 * 24)
            + ((*t).tm_hour as i64) * (60 * 60)
            + ((*t).tm_min as i64) * 60
            + (*t).tm_sec as i64) as ctypes::time_t
    }
}

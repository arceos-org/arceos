use core::ffi::{c_char, c_double, c_float, c_int};

macro_rules! strto_float_impl {
    ($type:ident, $s:expr, $endptr:expr) => {{
        let mut s = $s;
        let endptr = $endptr;

        // TODO: Handle named floats: NaN, Inf...

        while isspace(*s as c_int) {
            s = s.offset(1);
        }

        let mut result: $type = 0.0;
        let mut radix = 10;

        let result_sign = match *s as u8 {
            b'-' => {
                s = s.offset(1);
                -1.0
            }
            b'+' => {
                s = s.offset(1);
                1.0
            }
            _ => 1.0,
        };

        if *s as u8 == b'0' && *s.offset(1) as u8 == b'x' {
            s = s.offset(2);
            radix = 16;
        }

        while let Some(digit) = (*s as u8 as char).to_digit(radix) {
            result *= radix as $type;
            result += digit as $type;
            s = s.offset(1);
        }

        if *s as u8 == b'.' {
            s = s.offset(1);

            let mut i = 1.0;
            while let Some(digit) = (*s as u8 as char).to_digit(radix) {
                i *= radix as $type;
                result += digit as $type / i;
                s = s.offset(1);
            }
        }

        let s_before_exponent = s;

        let exponent = match (*s as u8, radix) {
            (b'e' | b'E', 10) | (b'p' | b'P', 16) => {
                s = s.offset(1);

                let is_exponent_positive = match *s as u8 {
                    b'-' => {
                        s = s.offset(1);
                        false
                    }
                    b'+' => {
                        s = s.offset(1);
                        true
                    }
                    _ => true,
                };

                // Exponent digits are always in base 10.
                if (*s as u8 as char).is_digit(10) {
                    let mut exponent_value = 0;

                    while let Some(digit) = (*s as u8 as char).to_digit(10) {
                        exponent_value *= 10;
                        exponent_value += digit;
                        s = s.offset(1);
                    }

                    let exponent_base = match radix {
                        10 => 10u128,
                        16 => 2u128,
                        _ => unreachable!(),
                    };

                    if is_exponent_positive {
                        Some(exponent_base.pow(exponent_value) as $type)
                    } else {
                        Some(1.0 / (exponent_base.pow(exponent_value) as $type))
                    }
                } else {
                    // Exponent had no valid digits after 'e'/'p' and '+'/'-', rollback
                    s = s_before_exponent;
                    None
                }
            }
            _ => None,
        };

        // Return pointer should be *mut
        if !endptr.is_null() {
            *endptr = s as *mut _;
        }

        if let Some(exponent) = exponent {
            result_sign * result * exponent
        } else {
            result_sign * result
        }
    }};
}

fn isspace(c: c_int) -> bool {
    c == c_int::from(b' ')
        || c == c_int::from(b'\t')
        || c == c_int::from(b'\n')
        || c == c_int::from(b'\r')
        || c == 0x0b
        || c == 0x0c
}

/// Convert a string to a double-precision number.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strtod(s: *const c_char, endptr: *mut *mut c_char) -> c_double {
    strto_float_impl!(c_double, s, endptr)
}

/// Convert a string to a float number.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strtof(s: *const c_char, endptr: *mut *mut c_char) -> c_float {
    strto_float_impl!(c_float, s, endptr)
}

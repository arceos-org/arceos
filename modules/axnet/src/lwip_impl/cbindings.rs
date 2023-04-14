use alloc::string::String;
use axlog::ax_print;
use core::ffi::{c_int, c_uchar};

#[no_mangle]
unsafe extern "C" fn lwip_print(str: *const c_uchar, mut args: ...) -> c_int {
    use printf_compat::{format, output};
    let mut s = String::new();
    let bytes_written = format(str, args.as_va_list(), output::fmt_write(&mut s));
    ax_print!("{}", s);
    bytes_written
}

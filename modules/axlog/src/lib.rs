#![cfg_attr(not(feature = "std"), no_std)]

extern crate log;

use core::fmt::{self, Write};
use core::str::FromStr;

use log::{Level, LevelFilter, Log, Metadata, Record};

#[cfg(not(feature = "std"))]
use crate_interface::call_interface;

pub use log::{debug, error, info, trace, warn};

#[macro_export]
macro_rules! ax_print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::__print_impl(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! ax_println {
    () => { ax_print!("\n") };
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::__print_impl(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

macro_rules! with_color {
    ($color_code:expr, $($arg:tt)*) => {{
        format_args!("\u{1B}[{}m{}\u{1B}[m", $color_code as u8, format_args!($($arg)*))
    }};
}

#[repr(u8)]
#[allow(dead_code)]
enum ColorCode {
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,
    BrightBlack = 90,
    BrightRed = 91,
    BrightGreen = 92,
    BrightYellow = 93,
    BrightBlue = 94,
    BrightMagenta = 95,
    BrightCyan = 96,
    BrightWhite = 97,
}

/// Extern interfaces called in this crate.
#[crate_interface::def_interface]
pub trait LogIf {
    /// write a string to the console.
    fn console_write_str(s: &str);

    /// get current time
    fn current_time() -> core::time::Duration;

    /// get current CPU ID.
    fn current_cpu_id() -> Option<usize>;

    /// get current task ID.
    fn current_task_id() -> Option<u64>;
}

struct Logger;

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        cfg_if::cfg_if! {
            if #[cfg(feature = "std")] {
                std::print!("{}", s);
            } else {
                call_interface!(LogIf::console_write_str, s);
            }
        }
        Ok(())
    }
}

impl Log for Logger {
    #[inline]
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level = record.level();
        let line = record.line().unwrap_or(0);
        let path = record.target();
        let args_color = match level {
            Level::Error => ColorCode::Red,
            Level::Warn => ColorCode::Yellow,
            Level::Info => ColorCode::Green,
            Level::Debug => ColorCode::Cyan,
            Level::Trace => ColorCode::BrightBlack,
        };

        cfg_if::cfg_if! {
            if #[cfg(feature = "std")] {
                __print_impl(with_color!(
                    ColorCode::White,
                    "[{time} {path}:{line}] {args}\n",
                    time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.6f"),
                    path = path,
                    line = line,
                    args = with_color!(args_color, "{}", record.args()),
                ));
            } else {
                let cpu_id = call_interface!(LogIf::current_cpu_id);
                let tid = call_interface!(LogIf::current_task_id);
                let now = call_interface!(LogIf::current_time);
                if let Some(cpu_id) = cpu_id {
                    if let Some(tid) = tid {
                        __print_impl(with_color!(
                            ColorCode::White,
                            "[{:>3}.{:06} {cpu_id}:{tid} {path}:{line}] {args}\n",
                            now.as_secs(),
                            now.subsec_micros(),
                            cpu_id = cpu_id,
                            tid = tid,
                            path = path,
                            line = line,
                            args = with_color!(args_color, "{}", record.args()),
                        ));
                    } else {
                        __print_impl(with_color!(
                            ColorCode::White,
                            "[{:>3}.{:06} {cpu_id} {path}:{line}] {args}\n",
                            now.as_secs(),
                            now.subsec_micros(),
                            cpu_id = cpu_id,
                            path = path,
                            line = line,
                            args = with_color!(args_color, "{}", record.args()),
                        ));
                    }
                } else {
                    __print_impl(with_color!(
                        ColorCode::White,
                        "[{:>3}.{:06} {path}:{line}] {args}\n",
                        now.as_secs(),
                        now.subsec_micros(),
                        path = path,
                        line = line,
                        args = with_color!(args_color, "{}", record.args()),
                    ));
                }
            }
        }
    }

    fn flush(&self) {}
}

#[doc(hidden)]
pub fn __print_impl(args: fmt::Arguments) {
    use spinlock::SpinNoIrq; // TODO: more efficient
    static LOCK: SpinNoIrq<()> = SpinNoIrq::new(());

    let _guard = LOCK.lock();
    Logger.write_fmt(args).unwrap();
}

pub fn init() {
    log::set_logger(&Logger).unwrap();
    log::set_max_level(LevelFilter::Warn);
}

pub fn set_max_level(level: &str) {
    let lf = LevelFilter::from_str(level)
        .ok()
        .unwrap_or(LevelFilter::Off);
    log::set_max_level(lf);
}

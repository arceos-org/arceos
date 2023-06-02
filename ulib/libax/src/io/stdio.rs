#[cfg(feature = "alloc")]
use alloc::string::String;

use crate::io::{prelude::*, BufReader, Result};
use crate::sync::Mutex;

struct StdinRaw;
struct StdoutRaw;

/// A handle to the standard input stream of a process.
pub struct Stdin {
    inner: &'static Mutex<BufReader<StdinRaw>>,
}

/// A handle to the global standard output stream of the current process.
pub struct Stdout {
    inner: &'static Mutex<StdoutRaw>,
}

impl StdinRaw {
    fn getchar() -> Option<u8> {
        axhal::console::getchar().map(|c| if c == b'\r' { b'\n' } else { c })
    }
}

impl Read for StdinRaw {
    // Non-blocking read, returns number of bytes read.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut read_len = 0;
        while read_len < buf.len() {
            if let Some(c) = Self::getchar() {
                buf[read_len] = c;
                read_len += 1;
            } else {
                break;
            }
        }
        Ok(read_len)
    }
}

impl Write for StdoutRaw {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        axhal::console::write_bytes(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result {
        Ok(())
    }
}

impl Stdin {
    /// Locks this handle and reads a line of input, appending it to the specified buffer.
    #[cfg(feature = "alloc")]
    pub fn read_line(&self, buf: &mut String) -> Result<usize> {
        self.inner.lock().read_line(buf)
    }

    #[allow(dead_code)]
    pub(crate) fn read_locked(&self, buf: &mut [u8]) -> Result<usize> {
        self.inner.lock().read(buf)
    }
}

impl Read for Stdin {
    // Block until at least one byte is read.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let read_len = self.inner.lock().read(buf)?;
        if buf.is_empty() || read_len > 0 {
            return Ok(read_len);
        }
        // try again until we get something
        loop {
            let read_len = self.inner.lock().read(buf)?;
            if read_len > 0 {
                return Ok(read_len);
            }
            crate::thread::yield_now();
        }
    }
}

impl Stdout {
    #[allow(dead_code)]
    pub(crate) fn write_locked(&self, buf: &[u8]) -> Result<usize> {
        self.inner.lock().write(buf)
    }
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.inner.lock().write(buf)
    }
    fn flush(&mut self) -> Result {
        self.inner.lock().flush()
    }
}

/// Constructs a new handle to the standard input of the current process.
pub fn stdin() -> Stdin {
    static INSTANCE: Mutex<BufReader<StdinRaw>> = Mutex::new(BufReader::new(StdinRaw));
    Stdin { inner: &INSTANCE }
}

/// Constructs a new handle to the standard output of the current process.
pub fn stdout() -> Stdout {
    static INSTANCE: Mutex<StdoutRaw> = Mutex::new(StdoutRaw);
    Stdout { inner: &INSTANCE }
}

/// Prints to the standard output.
///
/// Equivalent to the [`println!`] macro except that a newline is not printed at
/// the end of the message.
///
/// [`println!`]: crate::println
#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::io::__print_impl(format_args!($fmt $(, $($arg)+)?));
    }
}

/// Prints to the standard output, with a newline.
#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::io::__print_impl(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

#[doc(hidden)]
pub fn __print_impl(args: core::fmt::Arguments) {
    if cfg!(feature = "smp") {
        axlog::__print_impl(args); // synchronize using the lock in axlog
    } else {
        static INLINE_LOCK: Mutex<()> = Mutex::new(()); // not break in one line
        let _guard = INLINE_LOCK.lock();
        stdout().write_fmt(args).unwrap();
    }
}

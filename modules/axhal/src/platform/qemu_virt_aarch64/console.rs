use super::pl011::{console_getchar, console_putchar};

/// Writes a byte to the console.
pub fn putchar(c: u8) {
    match c {
        b'\n' => {
            console_putchar(b'\r');
            console_putchar(b'\n');
        }
        c => console_putchar(c),
    }
}

/// Reads a byte from the console, or returns [`None`] if no input is available.
pub fn getchar() -> Option<u8> {
    console_getchar()
}

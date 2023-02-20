use super::pl011::{console_getchar, console_putchar};

pub fn putchar(c: u8) {
    console_putchar(c);
}

pub fn getchar() -> Option<u8> {
    console_getchar()
}

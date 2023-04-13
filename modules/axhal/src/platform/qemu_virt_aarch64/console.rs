use super::pl011::{console_getchar, console_putchar};

pub fn putchar(c: u8) {
    match c {
        b'\n' => {
            console_putchar(b'\r');
            console_putchar(b'\n');
        }
        c => console_putchar(c),
    }
}

pub fn getchar() -> Option<u8> {
    console_getchar()
}

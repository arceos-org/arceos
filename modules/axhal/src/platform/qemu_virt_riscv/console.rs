pub fn putchar(c: u8) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c as usize);
}

pub fn getchar() -> Option<u8> {
    #[allow(deprecated)]
    match sbi_rt::legacy::console_getchar() as isize {
        -1 => None,
        c => Some(c as u8),
    }
}

#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate axlog;

#[cfg(not(test))]
mod lang_items;

const LOGO: &str = r#"
       d8888                            .d88888b.   .d8888b.
      d88888                           d88P" "Y88b d88P  Y88b
     d88P888                           888     888 Y88b.
    d88P 888 888d888  .d8888b  .d88b.  888     888  "Y888b.
   d88P  888 888P"   d88P"    d8P  Y8b 888     888     "Y88b.
  d88P   888 888     888      88888888 888     888       "888
 d8888888888 888     Y88b.    Y8b.     Y88b. .d88P Y88b  d88P
d88P     888 888      "Y8888P  "Y8888   "Y88888P"   "Y8888P"
"#;

extern "Rust" {
    fn main();
}

struct LogIfImpl;

impl axlog::LogIf for LogIfImpl {
    fn console_write_str(&self, s: &str) {
        use axhal::console::putchar;
        for c in s.chars() {
            match c {
                '\n' => {
                    putchar(b'\r');
                    putchar(b'\n');
                }
                _ => putchar(c as u8),
            }
        }
    }
}

#[cfg_attr(not(test), no_mangle)]
pub extern "C" fn rust_main() -> ! {
    axlog::set_interface(&LogIfImpl);
    println!("{}", LOGO);
    println!(
        "\
        arch = {}\n\
        platform = {}\n\
        build_mode = {}\n\
        log_level = {}\n\
        ",
        option_env!("ARCH").unwrap_or(""),
        option_env!("PLATFORM").unwrap_or(""),
        option_env!("MODE").unwrap_or(""),
        option_env!("LOG").unwrap_or(""),
    );

    axlog::init();
    axlog::set_max_level(option_env!("LOG").unwrap_or(""));
    info!("Logging is enabled.");

    #[cfg(feature = "alloc")]
    init_heap();

    unsafe { main() };

    axhal::misc::terminate()
}

#[cfg(feature = "alloc")]
fn init_heap() {
    const KERNEL_HEAP_LEN: usize = axconfig::KERNEL_HEAP_SIZE / core::mem::size_of::<u64>();
    static mut KERNEL_HEAP: [u64; KERNEL_HEAP_LEN] = [0; KERNEL_HEAP_LEN];
    axalloc::init(
        unsafe { KERNEL_HEAP.as_ptr() as usize },
        axconfig::KERNEL_HEAP_SIZE,
    );
}

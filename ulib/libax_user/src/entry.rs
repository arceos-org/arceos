use crate::task;

#[no_mangle]
#[link_section = ".text.start"]
extern "C" fn _start() {
    extern "Rust" {
        fn main();
    }
    super::logging::init();
    super::logging::set_max_level(option_env!("LOG").unwrap_or(""));
    super::allocate::init();
    unsafe {
        main();
    }
    task::exit(0);
}

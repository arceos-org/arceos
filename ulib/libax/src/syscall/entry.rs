use crate::task;



#[no_mangle]
fn __user_start() {
    extern "Rust" {
        fn main();
    }
    //super::logging::init();
    //super::logging::set_max_level(option_env!("LOG").unwrap_or(""));
    unsafe {
        main();
    }
    task::exit(0);
}

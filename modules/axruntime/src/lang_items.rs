use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);
    if cfg!(feature = "axbacktrace") {
        extern crate axbacktrace;
        axbacktrace::backtrace();
    }
    axhal::misc::terminate()
}

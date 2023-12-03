use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);
    #[cfg(feature = "backtrace")]
    {
        extern crate axbacktrace;
        axbacktrace::backtrace();
    }
    axhal::misc::terminate()
}

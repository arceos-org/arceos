use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);
    error!("{}", axbacktrace::Backtrace::capture());
    axhal::power::system_off()
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ax_println!("{}", info);
    ax_println!("{}", axbacktrace::Backtrace::capture());
    axhal::power::system_off()
}

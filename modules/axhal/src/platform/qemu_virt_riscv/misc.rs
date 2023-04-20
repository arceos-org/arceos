/// Shutdown the whole system, including all CPUs.
pub fn terminate() -> ! {
    info!("Shutting down...");
    sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::NoReason);
    unreachable!("It should shutdown!")
}

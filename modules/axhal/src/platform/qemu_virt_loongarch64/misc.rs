/// Shutdown the whole system, including all CPUs.
pub fn terminate() -> ! {
    info!("Shutting down...");
    crate::arch::halt();
    warn!("It should shutdown!");
    loop {
        crate::arch::halt();
    }
}

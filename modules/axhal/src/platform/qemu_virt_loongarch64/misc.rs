/// Shutdown the whole system, including all CPUs.
pub fn terminate() -> ! {
    loop {
        crate::arch::halt();
    }
}

#[cfg(feature = "smp")]
pub mod mp;

pub mod misc {
    pub fn terminate() -> ! {
        info!("Shutting down...");
        loop {
            crate::arch::halt();
        }
    }
}

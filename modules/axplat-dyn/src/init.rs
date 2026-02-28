use axplat::init::InitIf;

struct InitIfImpl;

#[impl_plat_interface]
impl InitIf for InitIfImpl {
    /// Initializes the platform at the early stage for the primary core.
    ///
    /// This function should be called immediately after the kernel has booted,
    /// and performed earliest platform configuration and initialization (e.g.,
    /// early console, clocking).
    fn init_early(cpu_id: usize, _dtb: usize) {
        axcpu::init::init_trap();
        somehal::timer::enable();
    }

    /// Initializes the platform at the early stage for secondary cores.
    #[cfg(feature = "smp")]
    fn init_early_secondary(cpu_id: usize) {
        axcpu::init::init_trap();
        somehal::timer::enable();
    }

    /// Initializes the platform at the later stage for the primary core.
    ///
    /// This function should be called after the kernel has done part of its
    /// initialization (e.g, logging, memory management), and finalized the rest of
    /// platform configuration and initialization.
    fn init_later(_cpu_id: usize, _dtb: usize) {
        somehal::post_paging();
        somehal::timer::irq_enable();
    }

    /// Initializes the platform at the later stage for secondary cores.
    #[cfg(feature = "smp")]
    fn init_later_secondary(_cpu_id: usize) {
        somehal::timer::irq_enable();
    }
}

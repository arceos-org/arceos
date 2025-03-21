mod boot;

pub mod console;
pub mod mem;
pub mod misc;
pub mod time;

#[cfg(feature = "irq")]
pub mod irq;

#[cfg(feature = "smp")]
pub mod mp;

unsafe extern "C" {
    fn rust_main(cpu_id: usize, dtb: usize);
    #[cfg(feature = "smp")]
    fn rust_main_secondary(cpu_id: usize);
}

unsafe extern "C" fn rust_entry(cpu_id: usize, dtb: usize) {
    crate::mem::clear_bss();
    crate::cpu::init_primary(cpu_id);
    #[cfg(feature = "uspace")]
    riscv::register::sstatus::set_sum();
    self::time::init_early();
    rust_main(cpu_id, dtb);
}

#[cfg(feature = "smp")]
unsafe extern "C" fn rust_entry_secondary(cpu_id: usize) {
    crate::cpu::init_secondary(cpu_id);
    #[cfg(feature = "uspace")]
    riscv::register::sstatus::set_sum();
    rust_main_secondary(cpu_id);
}

/// Initializes the platform devices for the primary CPU.
///
/// For example, the interrupt controller and the timer.
pub fn platform_init() {
    #[cfg(feature = "irq")]
    self::irq::init_percpu();
    self::time::init_percpu();
}

/// Initializes the platform devices for secondary CPUs.
#[cfg(feature = "smp")]
pub fn platform_init_secondary() {
    #[cfg(feature = "irq")]
    self::irq::init_percpu();
    self::time::init_percpu();
}

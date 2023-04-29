mod boot;
mod generic_timer;
mod pl011;
mod psci;

#[cfg(feature = "irq")]
mod gic;

#[cfg(feature = "smp")]
pub mod mp;

pub mod mem;

pub mod console {
    pub use super::pl011::*;
}

pub mod time {
    pub use super::generic_timer::*;
}

pub mod misc {
    pub use super::psci::system_off as terminate;
}

#[cfg(feature = "irq")]
pub mod irq {
    pub use super::gic::*;
}

extern "C" {
    fn exception_vector_base();
    fn rust_main(cpu_id: usize, dtb: usize);
    #[cfg(feature = "smp")]
    fn rust_main_secondary(cpu_id: usize);
}

unsafe extern "C" fn rust_entry(cpu_id: usize, dtb: usize) {
    crate::mem::clear_bss();
    crate::arch::set_exception_vector_base(exception_vector_base as usize);
    crate::cpu::init_primary(cpu_id);
    self::pl011::init();
    self::generic_timer::init_early();
    rust_main(cpu_id, dtb);
}

#[cfg(feature = "smp")]
unsafe extern "C" fn rust_entry_secondary(cpu_id: usize) {
    crate::arch::set_exception_vector_base(exception_vector_base as usize);
    crate::cpu::init_secondary(cpu_id);
    rust_main_secondary(cpu_id);
}

/// Initializes the platform devices for the primary CPU.
///
/// For example, the interrupt controller and the timer.
pub fn platform_init() {
    #[cfg(feature = "irq")]
    self::gic::init_primary();
    self::generic_timer::init_percpu();
}

/// Initializes the platform devices for secondary CPUs.
#[cfg(feature = "smp")]
pub fn platform_init_secondary() {
    #[cfg(feature = "irq")]
    self::gic::init_secondary();
    self::generic_timer::init_percpu();
}

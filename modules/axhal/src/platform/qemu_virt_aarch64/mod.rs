mod boot;
mod generic_timer;
mod pl011;
mod psci;

pub mod console;
pub mod mem;

#[cfg(feature = "irq")]
pub mod irq;

#[cfg(feature = "smp")]
pub mod mp;

pub mod time {
    pub use super::generic_timer::*;
}

pub mod misc {
    pub use super::psci::system_off as terminate;
}

extern "C" {
    fn exception_vector_base();
}

/// Initializes the platform for the primary CPU.
pub(crate) fn platform_init(cpu_id: usize, _dtb: *const u8) {
    crate::mem::clear_bss();
    crate::arch::set_exception_vector_base(exception_vector_base as usize);
    crate::cpu::init_percpu(cpu_id, true);

    #[cfg(feature = "irq")]
    self::irq::init();
    #[cfg(feature = "irq")]
    self::irq::init_percpu(cpu_id);

    self::pl011::init();
    self::generic_timer::init();
}

/// Initializes the platform for secondary CPUs.
#[cfg(feature = "smp")]
pub(crate) fn platform_init_secondary(cpu_id: usize, _dtb: *const u8) {
    crate::arch::set_exception_vector_base(exception_vector_base as usize);
    crate::cpu::init_percpu(cpu_id, false);
    #[cfg(feature = "irq")]
    self::irq::init_percpu(cpu_id);
    self::generic_timer::init_secondary();
}

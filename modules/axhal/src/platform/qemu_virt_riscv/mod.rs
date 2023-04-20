mod boot;

pub mod console;
pub mod irq;
pub mod mem;
pub mod misc;
pub mod time;

#[cfg(feature = "smp")]
pub mod mp;

extern "C" {
    fn trap_vector_base();
}

/// Initializes the platform for the primary CPU.
pub(crate) fn platform_init(cpu_id: usize, _dtb: usize) {
    crate::mem::clear_bss();
    crate::arch::set_trap_vector_base(trap_vector_base as usize);
    crate::cpu::init_percpu(cpu_id, true);
    self::irq::init();
    self::time::init();
}

/// Initializes the platform for secondary CPUs.
#[cfg(feature = "smp")]
pub(crate) fn platform_init_secondary(cpu_id: usize) {
    crate::arch::set_trap_vector_base(trap_vector_base as usize);
    crate::cpu::init_percpu(cpu_id, false);
    self::time::init();
}

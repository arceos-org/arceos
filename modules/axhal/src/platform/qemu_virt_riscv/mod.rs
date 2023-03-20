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

pub(crate) fn platform_init(_cpu_id: usize, _dtb: usize) {
    crate::mem::clear_bss();
    crate::arch::set_tap_vector_base(trap_vector_base as usize);
    self::irq::init();
    self::time::init();
}

#[cfg(feature = "smp")]
pub(crate) fn platform_init_secondary(_cpu_id: usize) {
    crate::arch::set_tap_vector_base(trap_vector_base as usize);
    self::time::init();
}

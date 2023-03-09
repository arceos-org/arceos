mod boot;

pub mod console;
pub mod irq;
pub mod mem;
pub mod misc;
pub mod time;

pub(crate) fn platform_init() {
    extern "C" {
        fn trap_vector_base();
    }
    crate::mem::clear_bss();
    crate::arch::set_tap_vector_base(trap_vector_base as usize);
    self::irq::init();
    self::time::init();
}

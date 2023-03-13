mod boot;
mod generic_timer;
mod pl011;

pub mod console;
pub mod irq;
pub mod mem;
pub mod misc;
pub mod mp;

pub mod time {
    pub use super::generic_timer::*;
}

pub(crate) fn platform_init(_dtb: usize) {
    extern "C" {
        fn exception_vector_base();
    }
    crate::mem::clear_bss();
    crate::arch::set_exception_vector_base(exception_vector_base as usize);
    self::irq::init();
    self::pl011::init();
    self::generic_timer::init();
    self::irq::init_percpu(0); // TODO
}

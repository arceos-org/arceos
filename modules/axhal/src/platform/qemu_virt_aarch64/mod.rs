mod boot;
mod generic_timer;
mod pl011;

pub mod console;
pub mod mem;
pub mod misc;

pub mod time {
    pub use super::generic_timer::{current_ticks, ticks_to_nanos};
}

pub fn platform_init() {
    extern "C" {
        fn exception_vector_base();
    }
    crate::arch::set_exception_vector_base(exception_vector_base as usize);
    generic_timer::init();
    pl011::init();
}

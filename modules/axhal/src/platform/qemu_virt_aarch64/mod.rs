pub mod console;
pub mod mem;
pub mod misc;

mod boot;
mod pl011;

pub fn platform_init() {
    extern "C" {
        fn exception_vector_base();
    }
    crate::arch::set_exception_vector_base(exception_vector_base as usize);
    pl011::init();
}

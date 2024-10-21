pub mod mem;
use core::ptr;

#[cfg(feature = "smp")]
pub mod mp;

#[cfg(feature = "irq")]
pub mod irq {
    pub use crate::platform::aarch64_common::gic::*;
}

pub mod console {
    pub use crate::platform::aarch64_common::pl011::*;
}

pub mod time {
    pub use crate::platform::aarch64_common::generic_timer::*;
}

pub mod misc {
    pub fn terminate() -> ! {
        info!("Shutting down...");
        loop {
            crate::arch::halt();
        }
    }
}

extern "C" {
    fn exception_vector_base();
    fn rust_main(cpu_id: usize, dtb: usize);
    #[cfg(feature = "smp")]
    fn rust_main_secondary(cpu_id: usize);
}

pub(crate) unsafe extern "C" fn rust_entry(cpu_id: usize, dtb: usize) {
    crate::mem::clear_bss();
    put_debug2();
    crate::arch::set_exception_vector_base(exception_vector_base as usize);
    put_debug2();
    crate::arch::write_page_table_root0(0.into()); // disable low address access
    put_debug_paged2();
    crate::cpu::init_primary(cpu_id);
    put_debug_paged2();
    super::aarch64_common::pl011::init_early();
    put_debug_paged2();
    super::aarch64_common::generic_timer::init_early();
    put_debug_paged2();
    rust_main(cpu_id, dtb);
}

#[cfg(all(target_arch = "aarch64"))]
#[no_mangle]
unsafe extern "C" fn put_debug2() {
    #[cfg(platform_family = "aarch64-phytium-pi")]
    {
        let state = (0x2800D018 as usize) as *mut u8;
        let put = (0x2800D000 as usize) as *mut u8;
        while (ptr::read_volatile(state) & (0x20 as u8)) != 0 {}
        *put = b'a';
    }
}

#[cfg(all(target_arch = "aarch64"))]
#[no_mangle]
unsafe extern "C" fn put_debug_paged2() {
    #[cfg(platform_family = "aarch64-phytium-pi")]
    {
        let state = (0xFFFF00002800D018 as usize) as *mut u8;
        let put = (0xFFFF00002800D000 as usize) as *mut u8;
        while (ptr::read_volatile(state) & (0x20 as u8)) != 0 {}
        *put = b'a';
    }
}

#[cfg(feature = "smp")]
pub(crate) unsafe extern "C" fn rust_entry_secondary(cpu_id: usize) {
    crate::arch::set_exception_vector_base(exception_vector_base as usize);
    crate::arch::write_page_table_root0(0.into()); // disable low address access
    crate::cpu::init_secondary(cpu_id);
    rust_main_secondary(cpu_id);
}

/// Initializes the platform devices for the primary CPU.
///
/// For example, the interrupt controller and the timer.
pub fn platform_init() {
    #[cfg(feature = "irq")]
    super::aarch64_common::gic::init_primary();
    super::aarch64_common::generic_timer::init_percpu();
    super::aarch64_common::pl011::init();
}

/// Initializes the platform devices for secondary CPUs.
#[cfg(feature = "smp")]
pub fn platform_init_secondary() {
    #[cfg(feature = "irq")]
    super::aarch64_common::gic::init_secondary();
    super::aarch64_common::generic_timer::init_percpu();
}

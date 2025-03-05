mod boot;

pub mod console;
#[cfg(feature = "irq")]
pub mod irq;
pub mod mem;
pub mod misc;
#[cfg(feature = "smp")]
pub mod mp;
pub mod time;

/// Initializes the platform devices for the primary CPU.
pub fn platform_init() {}

/// Initializes the platform devices for secondary CPUs.
#[cfg(feature = "smp")]
pub fn platform_init_secondary() {}

unsafe extern "C" {
    fn rust_main(cpu_id: usize, dtb: usize);
    #[cfg(feature = "smp")]
    fn rust_main_secondary(cpu_id: usize);
}

/// Rust temporary entry point
///
/// This function will be called after assembly boot stage.
unsafe extern "C" fn rust_entry(cpu_id: usize) {
    crate::mem::clear_bss();
    super::console::init_early();
    crate::cpu::init_primary(cpu_id);
    super::time::init_primary();
    super::time::init_percpu();

    unsafe {
        rust_main(cpu_id, 0);
    }
}

#[cfg(feature = "smp")]
/// The entry point for the second core.
pub(crate) extern "C" fn rust_entry_secondary(cpu_id: usize) {
    crate::cpu::init_secondary(cpu_id);
    super::time::init_percpu();

    unsafe {
        rust_main_secondary(cpu_id);
    }
}

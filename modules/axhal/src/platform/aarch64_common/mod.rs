mod boot;

#[cfg(not(platform_family = "aarch64-raspi"))]
pub mod psci;

pub mod mem;

pub mod console;

mod generic_timer;
pub mod time {
    pub use super::generic_timer::*;
}

#[cfg(feature = "irq")]
mod gic;
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

pub(crate) unsafe extern "C" fn rust_entry(cpu_id: usize, dtb: usize) {
    use crate::mem::phys_to_virt;

    crate::mem::clear_bss();
    crate::arch::set_exception_vector_base(exception_vector_base as usize);

    crate::arch::write_page_table_root0(0.into()); // disable low address access
    crate::cpu::init_primary(cpu_id);

    // init fdt
    crate::platform::mem::idmap_device(dtb);
    of::init_fdt_ptr(phys_to_virt(dtb.into()).as_usize() as *const u8);

    self::console::console_early_init();
    self::generic_timer::init_early();
    rust_main(cpu_id, dtb);
}

/// Initializes the platform devices for the primary CPU.
///
/// For example, the interrupt controller and the timer.
pub fn platform_init() {
    #[cfg(feature = "irq")]
    self::gic::init_primary();
    self::generic_timer::init_percpu();
    #[cfg(feature = "irq")]
    self::console::init_irq();
}

/// Initializes the platform devices for secondary CPUs.
#[cfg(feature = "smp")]
pub fn platform_init_secondary() {
    #[cfg(feature = "irq")]
    self::gic::init_secondary();
    self::generic_timer::init_percpu();
}

#[cfg(feature = "smp")]
pub(crate) unsafe extern "C" fn rust_entry_secondary(cpu_id: usize) {
    crate::arch::set_exception_vector_base(exception_vector_base as usize);
    crate::arch::write_page_table_root0(0.into()); // disable low address access
    crate::cpu::init_secondary(cpu_id);
    rust_main_secondary(cpu_id);
}

pub fn platform_name() -> &'static str {
    of::machin_name()
}

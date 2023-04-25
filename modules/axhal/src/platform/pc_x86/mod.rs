mod apic;
mod boot;
mod dtables;
mod uart16550;

pub mod mem;
pub mod misc;
pub mod time;

#[cfg(feature = "smp")]
pub mod mp;

#[cfg(feature = "irq")]
pub mod irq {
    pub use super::apic::*;
}

pub mod console {
    pub use super::uart16550::*;
}

extern "C" {
    fn rust_main(cpu_id: usize, dtb: usize) -> !;
}

fn current_cpu_id() -> usize {
    match raw_cpuid::CpuId::new().get_feature_info() {
        Some(finfo) => finfo.initial_local_apic_id() as usize,
        None => 0,
    }
}

unsafe extern "C" fn rust_entry(magic: usize, _mbi: usize) {
    // TODO: handle multiboot info
    if magic == self::boot::MULTIBOOT_BOOTLOADER_MAGIC {
        crate::mem::clear_bss();
        crate::cpu::init_primary(current_cpu_id());
        self::uart16550::init();
        self::dtables::init_primary();
        self::time::init_early();
        rust_main(current_cpu_id(), 0);
    }
}

/// Initializes the platform devices for the primary CPU.
pub fn platform_init() {
    self::apic::init_primary();
    self::time::init_primary();
}

/// Initializes the platform devices for secondary CPUs.
#[cfg(feature = "smp")]
pub fn platform_init_secondary() {
    self::apic::init_secondary();
    self::time::init_secondary();
}

mod dw_apb_uart;

pub mod mem;
pub mod misc;

#[cfg(feature = "smp")]
pub mod mp;

#[cfg(feature = "irq")]
pub mod irq {
    pub use crate::platform::aarch64_common::gic::*;
}

pub mod console {
    pub use super::dw_apb_uart::*;
}

pub mod time {
    pub use crate::platform::aarch64_common::generic_timer::*;
}

use crate::mp::CPU_HWID;
use crate::mp::MAX_HARTS;

extern "C" {
    fn exception_vector_base();
    fn rust_main(cpu_id: usize, dtb: usize);
    #[cfg(feature = "smp")]
    fn rust_main_secondary(cpu_id: usize);
}

pub(crate) unsafe extern "C" fn rust_entry(cpu_id: usize, dtb: usize) {
    crate::mem::clear_bss();
    crate::arch::set_exception_vector_base(exception_vector_base as usize);
    crate::cpu::init_primary(cpu_id);
    dw_apb_uart::init_early();
    super::aarch64_common::generic_timer::init_early();
    rust_main(cpu_id, dtb);
}

#[cfg(feature = "smp")]
pub(crate) unsafe extern "C" fn rust_entry_secondary(cpu_hwid: usize) {
    let mut cpu_id = cpu_hwid;
    let mut map_success = false;
    for index in 0..MAX_HARTS {
        if cpu_id == CPU_HWID[index] {
            cpu_id = index;
            map_success = true;
            break;
        }
    }
    if !map_success {
        panic!("CPU{} not found", cpu_id);
    }
    crate::arch::set_exception_vector_base(exception_vector_base as usize);
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
    #[cfg(feature = "irq")]
    dw_apb_uart::init_irq();
}

/// Initializes the platform devices for secondary CPUs.
#[cfg(feature = "smp")]
pub fn platform_init_secondary() {
    #[cfg(feature = "irq")]
    super::aarch64_common::gic::init_secondary();
    super::aarch64_common::generic_timer::init_percpu();
}

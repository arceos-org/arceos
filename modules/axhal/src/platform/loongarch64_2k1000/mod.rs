pub mod boot {
    pub use crate::platform::loongarch64_common::boot::*;
}
pub mod console {
    pub use crate::platform::loongarch64_common::console::*;
}
pub mod mem {
    pub use crate::platform::loongarch64_common::mem::*;
}

pub mod misc {
    pub use crate::platform::loongarch64_common::misc::*;
}
pub mod time {
    pub use crate::platform::loongarch64_common::time::*;
}

#[cfg(feature = "irq")]
pub mod irq {
    pub use crate::platform::loongarch64_common::irq::*;
}

#[cfg(feature = "smp")]
pub mod mp {
    pub use crate::platform::loongarch64_common::mp::*;
}

extern "C" {
    fn trap_vector_base();
    fn rust_main(cpu_id: usize, dtb: usize);
    fn _sbss();
    fn _ebss();
    #[cfg(feature = "smp")]
    fn rust_main_secondary(cpu_id: usize);
}

#[no_mangle]
pub(crate) unsafe extern "C" fn rust_entry(cpu_id: usize, _dtb: usize) {
    crate::mem::clear_bss();
    crate::cpu::init_primary(cpu_id);
    crate::arch::set_trap_vector_base(trap_vector_base as usize);
    rust_main(cpu_id, 0);
}

#[cfg(feature = "smp")]
pub(crate) unsafe extern "C" fn rust_entry_secondary(cpu_id: usize) {
    crate::arch::set_trap_vector_base(trap_vector_base as usize);
    crate::cpu::init_secondary(cpu_id);
    rust_main_secondary(cpu_id);
}

/// Initializes the platform devices for the primary CPU.
///
/// For example, the interrupt controller and external interrupts.
pub fn platform_init() {
    #[cfg(feature = "irq")]
    super::loongarch64_common::irq::init_percpu();
    super::loongarch64_common::time::init_percpu();
}

/// Initializes the platform devices for secondary CPUs.
#[cfg(feature = "smp")]
pub fn platform_init_secondary() {
    #[cfg(feature = "irq")]
    super::loongarch64_common::irq::init_percpu();
    super::loongarch64_common::time::init_percpu();
}

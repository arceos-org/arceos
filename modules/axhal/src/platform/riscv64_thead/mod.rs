mod boot;

#[path = "../riscv64_qemu_virt/console.rs"]
pub mod console;
#[path = "../riscv64_qemu_virt/mem.rs"]
pub mod mem;
#[path = "../riscv64_qemu_virt/misc.rs"]
pub mod misc;
#[path = "../riscv64_qemu_virt/time.rs"]
pub mod time;

#[cfg(feature = "irq")]
#[path = "../riscv64_qemu_virt/irq.rs"]
pub mod irq;

#[cfg(feature = "smp")]
#[path = "../riscv64_qemu_virt/mp.rs"]
pub mod mp;

extern "C" {
    fn trap_vector_base();
    fn rust_main(cpu_id: usize, dtb: usize);
    #[cfg(feature = "smp")]
    fn rust_main_secondary(cpu_id: usize);
}

unsafe extern "C" fn rust_entry(cpu_id: usize, dtb: usize) {
    crate::mem::clear_bss();
    crate::cpu::init_primary(cpu_id);
    crate::arch::set_trap_vector_base(trap_vector_base as usize);
    rust_main(cpu_id, dtb);
}

#[cfg(feature = "smp")]
unsafe extern "C" fn rust_entry_secondary(cpu_id: usize) {
    crate::arch::set_trap_vector_base(trap_vector_base as usize);
    crate::cpu::init_secondary(cpu_id);
    rust_main_secondary(cpu_id);
}

/// Initializes the platform devices for the primary CPU.
///
/// For example, the interrupt controller and the timer.
pub fn platform_init() {
    #[cfg(feature = "irq")]
    self::irq::init_percpu();
    self::time::init_percpu();
}

/// Initializes the platform devices for secondary CPUs.
#[cfg(feature = "smp")]
pub fn platform_init_secondary() {
    #[cfg(feature = "irq")]
    self::irq::init_percpu();
    self::time::init_percpu();
}

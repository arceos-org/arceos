//! [ArceOS] hardware abstraction layer, provides unified APIs for
//! platform-specific operations.
//!
//! It does the bootstrapping and initialization process for the specified
//! platform, and provides useful operations on the hardware.
//!
//! Currently supported platforms (specify by cargo features):
//!
//! - `x86-pc`: Standard PC with x86_64 ISA.
//! - `riscv64-qemu-virt`: QEMU virt machine with RISC-V ISA.
//! - `aarch64-qemu-virt`: QEMU virt machine with AArch64 ISA.
//! - `aarch64-raspi`: Raspberry Pi with AArch64 ISA.
//! - `dummy`: If none of the above platform is selected, the dummy platform
//!   will be used. In this platform, most of the operations are no-op or
//!   `unimplemented!()`. This platform is mainly used for [cargo test].
//!
//! # Cargo Features
//!
//! - `smp`: Enable SMP (symmetric multiprocessing) support.
//! - `fp-simd`: Enable floating-point and SIMD support.
//! - `paging`: Enable page table manipulation.
//! - `irq`: Enable interrupt handling support.
//! - `tls`: Enable kernel space thread-local storage support.
//! - `rtc`: Enable real-time clock support.
//! - `uspace`: Enable user space support.
//!
//! [ArceOS]: https://github.com/arceos-org/arceos
//! [cargo test]: https://doc.rust-lang.org/cargo/guide/tests.html

#![no_std]
#![feature(doc_auto_cfg)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[allow(unused_imports)]
#[macro_use]
extern crate axlog;

#[allow(unused_imports)]
#[macro_use]
extern crate memory_addr;

cfg_if::cfg_if! {
    if #[cfg(feature = "myplat")] {
        // link the custom platform crate in your application.
    } else if #[cfg(target_os = "none")] {
        #[cfg(target_arch = "x86_64")]
        extern crate axplat_x86_pc;
        #[cfg(target_arch = "aarch64")]
        extern crate axplat_aarch64_qemu_virt;
        #[cfg(target_arch = "riscv64")]
        extern crate axplat_riscv64_qemu_virt;
        #[cfg(target_arch = "loongarch64")]
        extern crate axplat_loongarch64_qemu_virt;
    } else {
        // Link the dummy platform implementation to pass cargo test.
        mod dummy;
    }
}

pub mod mem;
pub mod percpu;
pub mod time;

#[cfg(feature = "tls")]
pub mod tls;

#[cfg(feature = "irq")]
pub mod irq;

#[cfg(feature = "paging")]
pub mod paging;

/// Console input and output.
pub mod console {
    pub use axplat::console::{read_bytes, write_bytes};
}

/// CPU power management.
pub mod power {
    #[cfg(feature = "smp")]
    pub use axplat::power::cpu_boot;
    pub use axplat::power::system_off;
}

/// Trap handling.
pub mod trap {
    #[cfg(feature = "uspace")]
    pub use axcpu::trap::SYSCALL;
    pub use axcpu::trap::{IRQ, PAGE_FAULT};
    pub use axcpu::trap::{PageFaultFlags, register_trap_handler};
}

/// CPU register states for context switching.
///
/// There are three types of context:
///
/// - [`TaskContext`][axcpu::TaskContext]: The context of a task.
/// - [`TrapFrame`][axcpu::TrapFrame]: The context of an interrupt or an exception.
/// - [`UspaceContext`][axcpu::uspace::UspaceContext]: The context for user/kernel mode switching.
pub mod context {
    #[cfg(feature = "uspace")]
    pub use axcpu::uspace::UspaceContext;
    pub use axcpu::{TaskContext, TrapFrame};
}

pub use axcpu::asm;

#[cfg(feature = "smp")]
pub use axplat::init::{init_early_secondary, init_later_secondary};
#[cfg(feature = "smp")]
use core::sync::atomic::{AtomicUsize, Ordering};

/// Initializes CPU-local data structures for the primary core.
///
/// This function should be called as early as possible, as other initializations
/// may acess the CPU-local data.
pub fn init_percpu(cpu_id: usize) {
    self::percpu::init_primary(cpu_id);
}

/// Initializes CPU-local data structures for secondary cores.
///
/// This function should be called as early as possible, as other initializations
/// may acess the CPU-local data.
#[cfg(feature = "smp")]
pub fn init_percpu_secondary(cpu_id: usize) {
    self::percpu::init_secondary(cpu_id);
}

/// Initializes the platform and boot argument.
/// This function should be called as early as possible.
pub fn init_early(cpu_id: usize, arg: usize) {
    BOOT_ARG.init_once(arg);
    axplat::init::init_early(cpu_id, arg);
}

/// Initializes the platform later stage.
pub fn init_later(cpu_id: usize, arg: usize) {
    axplat::init::init_later(cpu_id, arg);
    init_cpu_num();
}

use lazyinit::LazyInit;

static BOOT_ARG: LazyInit<usize> = LazyInit::new();

/// Returns the boot argument.
/// This is typically the device tree blob address passed from the bootloader.
pub fn get_bootarg() -> usize {
    *BOOT_ARG
}

/// The number of CPUs in the system. Based on the number declared by the
/// platform crate and limited by the configured maximum CPU number.
#[cfg(feature = "smp")]
static CPU_NUM: AtomicUsize = AtomicUsize::new(1);

/// Gets the number of CPUs running in the system.
///
/// When SMP is disabled, this function always returns 1.
///
/// When SMP is enabled, it's the smaller one between the platform-declared CPU
/// number [`axplat::power::cpu_num`] and the configured maximum CPU number
/// `axconfig::plat::MAX_CPU_NUM`.
///
/// This value is determined during the BSP initialization phase.
pub fn cpu_num() -> usize {
    #[cfg(feature = "smp")]
    {
        // The BSP will always see the correct value because `CPU_NUM` is set by
        // itself.
        //
        // All APs will see the correct value because it is written with
        // `Ordering::Release` and read with `Ordering::Acquire`, ensuring
        // memory visibility.
        //
        // Acquire may result in a performance penalty, but this function is not
        // expected to be called frequently in normal operation.
        CPU_NUM.load(Ordering::Acquire)
    }
    #[cfg(not(feature = "smp"))]
    {
        1
    }
}

/// Initializes the CPU number information.
fn init_cpu_num() {
    #[cfg(feature = "smp")]
    {
        let plat_cpu_num = axplat::power::cpu_num();
        let max_cpu_num = axconfig::plat::MAX_CPU_NUM;
        let cpu_num = plat_cpu_num.min(max_cpu_num);

        info!("CPU number: max = {max_cpu_num}, platform = {plat_cpu_num}, use = {cpu_num}",);
        ax_println!("smp = {}", cpu_num); // for test purposes

        if plat_cpu_num > max_cpu_num {
            warn!(
                "platform declares more CPUs ({plat_cpu_num}) than configured max ({max_cpu_num}), \
                only the first {max_cpu_num} CPUs will be used."
            );
        }

        CPU_NUM.store(cpu_num, Ordering::Release);
    }
    #[cfg(not(feature = "smp"))]
    {
        ax_println!("smp = 1"); // for test purposes
    }
}

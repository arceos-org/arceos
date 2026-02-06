//! CPU-local data structures.

// Re-export platform percpu items only when a real platform is selected.
// The `percpu` module in `axplat` is only available in newer (unpublished) versions;
// the crates.io 0.3.0 release does not have it.
#[cfg(any(feature = "defplat", feature = "myplat"))]
pub use axplat::percpu::*;

/// Stub for single-CPU / dummy platform when not using defplat/myplat (e.g. for crates that
/// depend on axhal without a real platform, such as when building axipi for publish verification).
#[cfg(not(any(feature = "defplat", feature = "myplat")))]
#[inline]
pub fn this_cpu_id() -> usize {
    0
}

#[percpu::def_percpu]
static CURRENT_TASK_PTR: usize = 0;

/// Gets the pointer to the current task with preemption-safety.
///
/// Preemption may be enabled when calling this function. This function will
/// guarantee the correctness even the current task is preempted.
#[inline]
pub fn current_task_ptr<T>() -> *const T {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        // on x86, only one instruction is needed to read the per-CPU task pointer from `gs:[off]`.
        CURRENT_TASK_PTR.read_current_raw() as _
    }
    #[cfg(any(
        target_arch = "aarch64",
        target_arch = "riscv32",
        target_arch = "riscv64",
        target_arch = "loongarch64"
    ))]
    unsafe {
        // on RISC-V and LA64, reading `CURRENT_TASK_PTR` requires multiple instruction, so we disable local IRQs.
        let _guard = kernel_guard::IrqSave::new();
        CURRENT_TASK_PTR.read_current_raw() as _
    }
}

/// Sets the pointer to the current task with preemption-safety.
///
/// Preemption may be enabled when calling this function. This function will
/// guarantee the correctness even the current task is preempted.
///
/// # Safety
///
/// The given `ptr` must be pointed to a valid task structure.
#[inline]
pub unsafe fn set_current_task_ptr<T>(ptr: *const T) {
    #[cfg(target_arch = "x86_64")]
    {
        unsafe { CURRENT_TASK_PTR.write_current_raw(ptr as usize) }
    }
    #[cfg(any(
        target_arch = "aarch64",
        target_arch = "riscv32",
        target_arch = "riscv64",
        target_arch = "loongarch64"
    ))]
    {
        let _guard = kernel_guard::IrqSave::new();
        unsafe { CURRENT_TASK_PTR.write_current_raw(ptr as usize) }
    }
}

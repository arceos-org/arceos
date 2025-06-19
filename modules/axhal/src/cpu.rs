//! CPU-related operations.

#[percpu::def_percpu]
static CPU_ID: usize = 0;

#[percpu::def_percpu]
static IS_BSP: bool = false;

#[percpu::def_percpu]
static CURRENT_TASK_PTR: usize = 0;

/// Returns the ID of the current CPU.
#[inline]
pub fn this_cpu_id() -> usize {
    CPU_ID.read_current()
}

/// Returns whether the current CPU is the primary CPU (aka the bootstrap
/// processor or BSP)
#[inline]
pub fn this_cpu_is_bsp() -> bool {
    IS_BSP.read_current()
}

/// Caches the pointer to the current task in the `SP_EL0` register.
///
/// In aarch64 architecture, we use `SP_EL0` as the read cache for
/// the current task pointer. And this function will update this cache.
#[cfg(target_arch = "aarch64")]
unsafe fn cache_current_task_ptr<T>(ptr: *const T) {
    use aarch64_cpu::registers::{SP_EL0, Writeable};
    SP_EL0.set(ptr as u64);
}

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
        target_arch = "riscv32",
        target_arch = "riscv64",
        target_arch = "loongarch64"
    ))]
    unsafe {
        // on RISC-V and LA64, reading `CURRENT_TASK_PTR` requires multiple instruction, so we disable local IRQs.
        let _guard = kernel_guard::IrqSave::new();
        CURRENT_TASK_PTR.read_current_raw() as _
    }
    #[cfg(target_arch = "aarch64")]
    {
        // on ARM64, we use `SP_EL0` to cache the current task pointer.
        use aarch64_cpu::registers::{Readable, SP_EL0};
        let sp_el0 = SP_EL0.get();
        if sp_el0 != 0 {
            // use the cached value
            sp_el0 as _
        } else {
            // `SP_EL0` will be reset to 0 on each interrupt or exception,
            // call the slow path and cache the value.
            unsafe {
                let _guard = kernel_guard::IrqSave::new();
                let ptr = CURRENT_TASK_PTR.read_current_raw() as _;
                cache_current_task_ptr(ptr);
                ptr
            }
        }
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
        target_arch = "riscv32",
        target_arch = "riscv64",
        target_arch = "loongarch64"
    ))]
    {
        let _guard = kernel_guard::IrqSave::new();
        unsafe { CURRENT_TASK_PTR.write_current_raw(ptr as usize) }
    }
    #[cfg(target_arch = "aarch64")]
    {
        let _guard = kernel_guard::IrqSave::new();
        unsafe {
            CURRENT_TASK_PTR.write_current_raw(ptr as usize);
            cache_current_task_ptr(ptr);
        }
    }
}

#[allow(dead_code)]
pub(crate) fn init_primary(cpu_id: usize) {
    percpu::init();
    percpu::init_percpu_reg(cpu_id);
    unsafe {
        CPU_ID.write_current_raw(cpu_id);
        IS_BSP.write_current_raw(true);
    }
}

#[allow(dead_code)]
pub(crate) fn init_secondary(cpu_id: usize) {
    percpu::init_percpu_reg(cpu_id);
    unsafe {
        CPU_ID.write_current_raw(cpu_id);
        IS_BSP.write_current_raw(false);
    }
}

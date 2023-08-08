const fn align_up(val: usize) -> usize {
    const PAGE_SIZE: usize = 0x1000;
    (val + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

#[cfg(not(target_os = "none"))]
static PERCPU_AREA_BASE: spin::once::Once<usize> = spin::once::Once::new();

/// Returns the per-CPU data area size for one CPU.
#[doc(cfg(not(feature = "sp-naive")))]
pub fn percpu_area_size() -> usize {
    extern "C" {
        fn _percpu_load_start();
        fn _percpu_load_end();
    }
    use percpu_macros::percpu_symbol_offset;
    percpu_symbol_offset!(_percpu_load_end) - percpu_symbol_offset!(_percpu_load_start)
}

/// Returns the base address of the per-CPU data area on the given CPU.
///
/// if `cpu_id` is 0, it returns the base address of all per-CPU data areas.
#[doc(cfg(not(feature = "sp-naive")))]
pub fn percpu_area_base(cpu_id: usize) -> usize {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "none")] {
            extern "C" {
                fn _percpu_start();
            }
            let base = _percpu_start as usize;
        } else {
            let base = *PERCPU_AREA_BASE.get().unwrap();
        }
    }
    base + cpu_id * align_up(percpu_area_size())
}

/// Initialize the per-CPU data area for `max_cpu_num` CPUs.
pub fn init(max_cpu_num: usize) {
    let size = percpu_area_size();

    #[cfg(target_os = "linux")]
    {
        // we not load the percpu section in ELF, allocate them here.
        let total_size = align_up(size) * max_cpu_num;
        let layout = std::alloc::Layout::from_size_align(total_size, 0x1000).unwrap();
        PERCPU_AREA_BASE.call_once(|| unsafe { std::alloc::alloc(layout) as usize });
    }

    let base = percpu_area_base(0);
    for i in 1..max_cpu_num {
        let secondary_base = percpu_area_base(i);
        // copy per-cpu data of the primary CPU to other CPUs.
        unsafe {
            core::ptr::copy_nonoverlapping(base as *const u8, secondary_base as *mut u8, size);
        }
    }
}

/// Read the architecture-specific thread pointer register on the current CPU.
pub fn get_local_thread_pointer() -> usize {
    let tp;
    unsafe {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "x86_64")] {
                tp = if cfg!(target_os = "linux") {
                    SELF_PTR.read_current_raw()
                } else if cfg!(target_os = "none") {
                    x86::msr::rdmsr(x86::msr::IA32_GS_BASE) as usize
                } else {
                    unimplemented!()
                };
            } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
                core::arch::asm!("mv {}, gp", out(reg) tp)
            } else if #[cfg(target_arch = "aarch64")] {
                core::arch::asm!("mrs {}, TPIDR_EL1", out(reg) tp)
            }
        }
    }
    tp
}

/// Set the architecture-specific thread pointer register to the per-CPU data
/// area base on the current CPU.
///
/// `cpu_id` indicates which per-CPU data area to use.
pub fn set_local_thread_pointer(cpu_id: usize) {
    let tp = percpu_area_base(cpu_id);
    unsafe {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "x86_64")] {
                if cfg!(target_os = "linux") {
                    const ARCH_SET_GS: u32 = 0x1001;
                    const SYS_ARCH_PRCTL: u32 = 158;
                    core::arch::asm!(
                        "syscall",
                        in("eax") SYS_ARCH_PRCTL,
                        in("edi") ARCH_SET_GS,
                        in("rsi") tp,
                    );
                } else if cfg!(target_os = "none") {
                    x86::msr::wrmsr(x86::msr::IA32_GS_BASE, tp as u64);
                } else {
                    unimplemented!()
                }
                SELF_PTR.write_current_raw(tp);
            } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
                core::arch::asm!("mv gp, {}", in(reg) tp)
            } else if #[cfg(target_arch = "aarch64")] {
                core::arch::asm!("msr TPIDR_EL1, {}", in(reg) tp)
            }
        }
    }
}

/// To use `percpu::__priv::NoPreemptGuard::new()` in macro expansion.
#[allow(unused_imports)]
#[cfg(feature = "preempt")]
use crate as percpu;

/// On x86, we use `gs:SELF_PTR` to store the address of the per-CPU data area base.
#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[percpu_macros::def_percpu]
static SELF_PTR: usize = 0;

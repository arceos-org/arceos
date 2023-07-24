//! Functions to read time stamp counters on x86.
use core::arch::asm;

use crate::arch::_rdtsc;

/// Read the time stamp counter.
///
/// The RDTSC instruction is not a serializing instruction.
/// It does not necessarily wait until all previous instructions
/// have been executed before reading the counter. Similarly,
/// subsequent instructions may begin execution before the
/// read operation is performed. If software requires RDTSC to be
/// executed only after all previous instructions have completed locally,
/// it can either use RDTSCP or execute the sequence LFENCE;RDTSC.
///
/// # Safety
/// * Causes a GP fault if the TSD flag in register CR4 is set and the CPL
///   is greater than 0.
pub unsafe fn rdtsc() -> u64 {
    _rdtsc() as u64
}

/// Read the time stamp counter.
///
/// The RDTSCP instruction waits until all previous instructions have been
/// executed before reading the counter. However, subsequent instructions may
/// begin execution before the read operation is performed.
///
/// Volatile is used here because the function may be used to act as an
/// instruction barrier.
///
/// # Returns
/// - The current time stamp counter value of the CPU as a `u64`.
/// - The contents of `IA32_TSC_AUX` on that particular core. This is an OS
///   defined value. For example, Linux writes `numa_id << 12 | core_id` into
///   it. See also [`crate::rdpid`].
///
/// # Note
/// One can use `core::arch::x86_64::__rdtscp` from the Rust core library as
/// well. We don't rely on it because it only returns the time-stamp counter of
/// rdtscp and not the contents of `IA32_TSC_AUX`.
///
/// # Safety
/// * Causes a GP fault if the TSD flag in register CR4 is set and the CPL is
///   greater than 0.
pub unsafe fn rdtscp() -> (u64, u32) {
    let eax: u32;
    let ecx: u32;
    let edx: u32;
    asm!(
      "rdtscp",
      lateout("eax") eax,
      lateout("ecx") ecx,
      lateout("edx") edx,
      options(nomem, nostack)
    );

    let counter: u64 = (edx as u64) << 32 | eax as u64;
    (counter, ecx)
}

#[cfg(all(test, feature = "utest"))]
mod test {
    use super::*;

    #[test]
    fn check_rdtsc() {
        let cpuid = crate::cpuid::CpuId::new();
        let has_tsc = cpuid
            .get_feature_info()
            .map_or(false, |finfo| finfo.has_tsc());

        if has_tsc {
            unsafe {
                assert!(rdtsc() > 0, "rdtsc returned 0, unlikely!");
            }
        }
    }

    #[test]
    fn check_rdtscp() {
        let cpuid = crate::cpuid::CpuId::new();
        let has_rdtscp = cpuid
            .get_extended_processor_and_feature_identifiers()
            .map_or(false, |einfo| einfo.has_rdtscp());

        if has_rdtscp {
            unsafe {
                // Check cycle counter:
                assert!(rdtscp().0 > 0, "rdtscp returned 0, unlikely!");

                // Check TSC AUX is correct (currently when using Linux only):
                // See also: https://elixir.bootlin.com/linux/v5.18.8/source/arch/x86/include/asm/segment.h#L241
                if cfg!(target_os = "linux") {
                    let mut cpu: u32 = 0;
                    let mut node: u32 = 0;
                    libc::syscall(libc::SYS_getcpu, &mut cpu, &mut node, 0);
                    assert_eq!(
                        rdtscp().1,
                        node << 12 | cpu,
                        "rdtscp AUX didn't match getcpu call!"
                    );
                }
            }
        }
    }
}

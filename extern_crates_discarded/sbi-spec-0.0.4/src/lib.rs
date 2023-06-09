//! RISC-V SBI Specification structure and constant definitions
//!
//! This crate adapts to RISC-V SBI Specification verion 1.0.0 ratified.
//! It provides structures in Rust semantics and best practices to simplify
//! designs of RISC-V SBI ecosystem, both implementation and applications.
//!
//! You may find it convenient to use this library in a vast range of packages,
//! from operating system kernels, hypervisors, to SBI bare metal implementations.
//! This crate is `no_std` compatible and does not need dymanic memory allocation,
//! which make it suitable for embedded development.
//!
//! Although this library is dedicated to RISC-V architecture, it does not limit
//! which build target the dependents should compile into. For example, you are
//! developing a RISC-V emulator on platforms other than RISC-V, the emulator
//! designed on other platforms can still make use of `sbi-spec` structures to
//! provide necessary features the emulated RISC-V environment would make use of.
#![no_std]
#![deny(missing_docs, unsafe_code, unstable_features)]

// §3
pub mod binary;
// §4
pub mod base;
// §5
#[cfg(feature = "legacy")]
pub mod legacy;
// §6
pub mod time;
// §7
pub mod spi;
// §8
pub mod rfnc;
// §9
pub mod hsm;
// §10
pub mod srst;
// §11
pub mod pmu;

/// Converts SBI EID from str.
const fn eid_from_str(name: &str) -> i32 {
    match *name.as_bytes() {
        [a] => i32::from_be_bytes([0, 0, 0, a]),
        [a, b] => i32::from_be_bytes([0, 0, a, b]),
        [a, b, c] => i32::from_be_bytes([0, a, b, c]),
        [a, b, c, d] => i32::from_be_bytes([a, b, c, d]),
        _ => unreachable!(),
    }
}

/// Checks during compilation, and provides an item list for developers.
#[cfg(test)]
mod tests {
    use static_assertions::{
        assert_eq_align, assert_eq_size, assert_fields, assert_impl_all, const_assert_eq,
    };
    // §3
    #[test]
    fn test_binary() {
        use crate::binary::*;
        assert_eq_align!(SbiRet, usize);
        assert_eq_size!(SbiRet, [usize; 2]);
        assert_fields!(SbiRet: error);
        assert_fields!(SbiRet: value);
        assert_impl_all!(SbiRet: Copy, Clone, PartialEq, Eq, core::fmt::Debug);

        const_assert_eq!(0, RET_SUCCESS as isize);
        const_assert_eq!(-1, RET_ERR_FAILED as isize);
        const_assert_eq!(-2, RET_ERR_NOT_SUPPORTED as isize);
        const_assert_eq!(-3, RET_ERR_INVALID_PARAM as isize);
        const_assert_eq!(-4, RET_ERR_DENIED as isize);
        const_assert_eq!(-5, RET_ERR_INVALID_ADDRESS as isize);
        const_assert_eq!(-6, RET_ERR_ALREADY_AVAILABLE as isize);
        const_assert_eq!(-7, RET_ERR_ALREADY_STARTED as isize);
        const_assert_eq!(-8, RET_ERR_ALREADY_STOPPED as isize);
    }
    // §4
    #[test]
    fn test_base() {
        use crate::base::*;
        const_assert_eq!(0x10, EID_BASE);
        const_assert_eq!(0, GET_SBI_SPEC_VERSION);
        const_assert_eq!(1, GET_SBI_IMPL_ID);
        const_assert_eq!(2, GET_SBI_IMPL_VERSION);
        const_assert_eq!(3, PROBE_EXTENSION);
        const_assert_eq!(4, GET_MVENDORID);
        const_assert_eq!(5, GET_MARCHID);
        const_assert_eq!(6, GET_MIMPID);
        const_assert_eq!(0, impl_id::BBL);
        const_assert_eq!(1, impl_id::OPEN_SBI);
        const_assert_eq!(2, impl_id::XVISOR);
        const_assert_eq!(3, impl_id::KVM);
        const_assert_eq!(4, impl_id::RUST_SBI);
        const_assert_eq!(5, impl_id::DIOSIX);
        const_assert_eq!(6, impl_id::COFFER);
    }
    // §5
    #[cfg(feature = "legacy")]
    #[test]
    fn test_legacy() {
        use crate::legacy::*;
        const_assert_eq!(0, LEGACY_SET_TIMER);
        const_assert_eq!(1, LEGACY_CONSOLE_PUTCHAR);
        const_assert_eq!(2, LEGACY_CONSOLE_GETCHAR);
        const_assert_eq!(3, LEGACY_CLEAR_IPI);
        const_assert_eq!(4, LEGACY_SEND_IPI);
        const_assert_eq!(5, LEGACY_REMOTE_FENCE_I);
        const_assert_eq!(6, LEGACY_REMOTE_SFENCE_VMA);
        const_assert_eq!(7, LEGACY_REMOTE_SFENCE_VMA_ASID);
        const_assert_eq!(8, LEGACY_SHUTDOWN);
    }
    // §6
    #[test]
    fn test_time() {
        use crate::time::*;
        const_assert_eq!(0x54494D45, EID_TIME);
        const_assert_eq!(0, SET_TIMER);
    }
    // §7
    #[test]
    fn test_spi() {
        use crate::spi::*;
        const_assert_eq!(0x735049, EID_SPI);
        const_assert_eq!(0, SEND_IPI);
    }
    // §8
    #[test]
    fn test_rfnc() {
        use crate::rfnc::*;
        const_assert_eq!(0x52464E43, EID_RFNC);
        const_assert_eq!(0, REMOTE_FENCE_I);
        const_assert_eq!(1, REMOTE_SFENCE_VMA);
        const_assert_eq!(2, REMOTE_SFENCE_VMA_ASID);
        const_assert_eq!(3, REMOTE_HFENCE_GVMA_VMID);
        const_assert_eq!(4, REMOTE_HFENCE_GVMA);
        const_assert_eq!(5, REMOTE_HFENCE_VVMA_ASID);
        const_assert_eq!(6, REMOTE_HFENCE_VVMA);
    }
    // §9
    #[test]
    fn test_hsm() {
        use crate::hsm::*;
        const_assert_eq!(0x48534D, EID_HSM);
        const_assert_eq!(0, HART_STATE_STARTED);
        const_assert_eq!(1, HART_STATE_STOPPED);
        const_assert_eq!(2, HART_STATE_START_PENDING);
        const_assert_eq!(3, HART_STATE_STOP_PENDING);
        const_assert_eq!(4, HART_STATE_SUSPENDED);
        const_assert_eq!(5, HART_STATE_SUSPEND_PENDING);
        const_assert_eq!(6, HART_STATE_RESUME_PENDING);
        const_assert_eq!(0x0000_0000, HART_SUSPEND_TYPE_RETENTIVE);
        const_assert_eq!(0x8000_0000, HART_SUSPEND_TYPE_NON_RETENTIVE);
        const_assert_eq!(0, HART_START);
        const_assert_eq!(1, HART_STOP);
        const_assert_eq!(2, HART_GET_STATUS);
        const_assert_eq!(3, HART_SUSPEND);
    }
    // §10
    #[test]
    fn test_srst() {
        use crate::srst::*;
        const_assert_eq!(0x53525354, EID_SRST);
        const_assert_eq!(0, RESET_TYPE_SHUTDOWN);
        const_assert_eq!(1, RESET_TYPE_COLD_REBOOT);
        const_assert_eq!(2, RESET_TYPE_WARM_REBOOT);
        const_assert_eq!(0, RESET_REASON_NO_REASON);
        const_assert_eq!(1, RESET_REASON_SYSTEM_FAILURE);
        const_assert_eq!(0, SYSTEM_RESET);
    }
    // §11
    #[test]
    fn test_pmu() {
        use crate::pmu::*;
        const_assert_eq!(0x504D55, EID_PMU);
        const_assert_eq!(0, PMU_NUM_COUNTERS);
        const_assert_eq!(1, PMU_COUNTER_GET_INFO);
        const_assert_eq!(2, PMU_COUNTER_CONFIG_MATCHING);
        const_assert_eq!(3, PMU_COUNTER_START);
        const_assert_eq!(4, PMU_COUNTER_STOP);
        const_assert_eq!(5, PMU_COUNTER_FW_READ);
    }
}

//! ARM Power State Coordination Interface.

#![allow(dead_code)]

use axconfig::devices::PSCI_METHOD;

pub const PSCI_0_2_FN_BASE: u32 = 0x84000000;
pub const PSCI_0_2_64BIT: u32 = 0x40000000;
pub const PSCI_0_2_FN_CPU_SUSPEND: u32 = PSCI_0_2_FN_BASE + 1;
pub const PSCI_0_2_FN_CPU_OFF: u32 = PSCI_0_2_FN_BASE + 2;
pub const PSCI_0_2_FN_CPU_ON: u32 = PSCI_0_2_FN_BASE + 3;
pub const PSCI_0_2_FN_MIGRATE: u32 = PSCI_0_2_FN_BASE + 5;
pub const PSCI_0_2_FN_SYSTEM_OFF: u32 = PSCI_0_2_FN_BASE + 8;
pub const PSCI_0_2_FN_SYSTEM_RESET: u32 = PSCI_0_2_FN_BASE + 9;
pub const PSCI_0_2_FN64_CPU_SUSPEND: u32 = PSCI_0_2_FN_BASE + PSCI_0_2_64BIT + 1;
pub const PSCI_0_2_FN64_CPU_ON: u32 = PSCI_0_2_FN_BASE + PSCI_0_2_64BIT + 3;
pub const PSCI_0_2_FN64_MIGRATE: u32 = PSCI_0_2_FN_BASE + PSCI_0_2_64BIT + 5;

/// PSCI return values, inclusive of all PSCI versions.
#[derive(PartialEq, Debug)]
#[repr(i32)]
pub enum PsciError {
    NotSupported = -1,
    InvalidParams = -2,
    Denied = -3,
    AlreadyOn = -4,
    OnPending = -5,
    InternalFailure = -6,
    NotPresent = -7,
    Disabled = -8,
    InvalidAddress = -9,
}

impl From<i32> for PsciError {
    fn from(code: i32) -> PsciError {
        use PsciError::*;
        match code {
            -1 => NotSupported,
            -2 => InvalidParams,
            -3 => Denied,
            -4 => AlreadyOn,
            -5 => OnPending,
            -6 => InternalFailure,
            -7 => NotPresent,
            -8 => Disabled,
            -9 => InvalidAddress,
            _ => panic!("Unknown PSCI error code: {}", code),
        }
    }
}

/// arm,psci method: smc
/// when SMCCC_CONDUIT_SMC = 1
fn arm_smccc_smc(func: u32, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        core::arch::asm!(
            "smc #0",
            inlateout("x0") func as usize => ret,
            in("x1") arg0,
            in("x2") arg1,
            in("x3") arg2,
        )
    }
    ret
}

/// psci "hvc" method call
fn psci_hvc_call(func: u32, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let ret;
    unsafe {
        core::arch::asm!(
            "hvc #0",
            inlateout("x0") func as usize => ret,
            in("x1") arg0,
            in("x2") arg1,
            in("x3") arg2,
        )
    }
    ret
}

fn psci_call(func: u32, arg0: usize, arg1: usize, arg2: usize) -> Result<(), PsciError> {
    let ret = match PSCI_METHOD {
        "smc" => arm_smccc_smc(func, arg0, arg1, arg2),
        "hvc" => psci_hvc_call(func, arg0, arg1, arg2),
        _ => panic!("Unknown PSCI method: {}", PSCI_METHOD),
    };
    if ret == 0 {
        Ok(())
    } else {
        Err(PsciError::from(ret as i32))
    }
}

/// Shutdown the whole system, including all CPUs.
pub fn system_off() -> ! {
    info!("Shutting down...");
    psci_call(PSCI_0_2_FN_SYSTEM_OFF, 0, 0, 0).ok();
    warn!("It should shutdown!");
    loop {
        crate::arch::halt();
    }
}

/// Power up a core. This call is used to power up cores that either:
///
/// * Have not yet been booted into the calling supervisory software.
/// * Have been previously powered down with a `cpu_off` call.
///
/// `target_cpu` contains a copy of the affinity fields of the MPIDR register.
/// `entry_point` is the physical address of the secondary CPU's entry point.
/// `arg` will be passed to the `X0` register of the secondary CPU.
pub fn cpu_on(target_cpu: usize, entry_point: usize, arg: usize) {
    info!("Starting CPU {:x} ON ...", target_cpu);
    let res = psci_call(PSCI_0_2_FN64_CPU_ON, target_cpu, entry_point, arg);
    if let Err(e) = res {
        error!("failed to boot CPU {:x} ({:?})", target_cpu, e);
    }
}

/// Power down the calling core. This call is intended for use in hotplug. A
/// core that is powered down by `cpu_off` can only be powered up again in
/// response to a `cpu_on`.
pub fn cpu_off() {
    const PSCI_POWER_STATE_TYPE_STANDBY: u32 = 0;
    const PSCI_POWER_STATE_TYPE_POWER_DOWN: u32 = 1;
    const PSCI_0_2_POWER_STATE_TYPE_SHIFT: u32 = 16;
    let state: u32 = PSCI_POWER_STATE_TYPE_POWER_DOWN << PSCI_0_2_POWER_STATE_TYPE_SHIFT;
    psci_call(PSCI_0_2_FN_CPU_OFF, state as usize, 0, 0).ok();
}

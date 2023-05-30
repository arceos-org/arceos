#![allow(dead_code)]

//! ARM Power State Coordination Interface.

use core::arch::asm;

const PSCI_CPU_ON: u32 = 0x8400_0003;
const PSCI_SYSTEM_OFF: u32 = 0x8400_0008;

fn psci_hvc_call(func: u32, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let ret;
    unsafe {
        asm!(
            "hvc #0",
            inlateout("x0") func as usize => ret,
            in("x1") arg0,
            in("x2") arg1,
            in("x3") arg2,
        )
    }
    ret
}

/// Shutdown the whole system, including all CPUs.
pub fn system_off() -> ! {
    info!("Shutting down...");
    psci_hvc_call(PSCI_SYSTEM_OFF, 0, 0, 0);
    warn!("It should shutdown!");
    loop {
        crate::arch::halt();
    }
}

/// Starts a secondary CPU with the given ID.
///
/// When the CPU is started, it will jump to the given entry and set the
/// corresponding register to the given argument.
pub fn cpu_on(id: usize, entry: usize, arg: usize) {
    debug!("Starting core {}...", id);
    assert_eq!(psci_hvc_call(PSCI_CPU_ON, id, entry, arg), 0);
    debug!("Started core {}!", id);
}

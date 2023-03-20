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

pub fn terminate() -> ! {
    info!("Shutting down...");
    psci_hvc_call(PSCI_SYSTEM_OFF, 0, 0, 0);
    unreachable!("It should shutdown!")
}

pub fn start(id: usize, entry: usize, arg: usize) {
    info!("Starting core {}...", id);
    assert_eq!(psci_hvc_call(PSCI_CPU_ON, id, entry, arg), 0);
    info!("Started core {}!", id);
}

use crate::mem::{va, virt_to_phys, PhysAddr, VirtAddr};

/// Starts the given secondary CPU with its boot stack.
pub fn start_secondary_cpu(cpu_id: usize, stack_top: PhysAddr) {
    extern "C" {
        fn _start_secondary();
    }
    let entry = virt_to_phys(va!(_start_secondary as usize));
    crate::platform::aarch64_common::psci::cpu_on(cpu_id, entry.as_usize(), stack_top.as_usize());
}

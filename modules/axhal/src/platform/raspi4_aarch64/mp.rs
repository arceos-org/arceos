use crate::mem::{virt_to_phys, PhysAddr, VirtAddr};

/// Starts the given secondary CPU with its boot stack.
pub fn start_secondary_cpu(cpu_id: usize, stack_top: PhysAddr) {
    extern "C" {
        fn _start_secondary();
    }
    let entry = virt_to_phys(VirtAddr::from(_start_secondary as usize));
    super::psci::cpu_on(cpu_id, entry.as_usize(), stack_top.as_usize());
}

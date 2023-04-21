use crate::mem::PhysAddr;

/// Start a secondary CPU with the given ID.
///
/// When the CPU is started, it will jump to the given `entry` and set the
/// stack pointer registers to `stack_top`.
pub fn start_secondary_cpu(cpu_id: usize, entry: PhysAddr, stack_top: PhysAddr) {
    super::psci::cpu_on(cpu_id, entry.as_usize(), stack_top.as_usize());
}

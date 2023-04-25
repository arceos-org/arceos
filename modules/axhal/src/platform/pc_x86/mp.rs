use crate::mem::PhysAddr;

/// Starts the given secondary CPU with its boot stack.
pub fn start_secondary_cpu(_cpu_id: usize, _stack_top: PhysAddr) {}

use crate::mem::PhysAddr;

pub fn start_secondary_cpu(cpu_id: usize, entry: PhysAddr, args: PhysAddr) {
    super::psci::cpu_on(cpu_id, entry.as_usize(), args.as_usize());
}

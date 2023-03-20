use super::misc::start;
use crate::mem::PhysAddr;

pub fn start_secondary_cpu(cpu_id: usize, entry: PhysAddr, args: PhysAddr) {
    start(cpu_id, entry.as_usize(), args.as_usize());
}

use crate::mem::PhysAddr;

/// Starts a secondary CPU with the given hart ID.
///
/// When the CPU is started, it will jump to the given `entry` and set the
/// stack pointer registers to `stack_top`.
pub fn start_secondary_cpu(hartid: usize, entry: PhysAddr, stack_top: PhysAddr) {
    if sbi_rt::probe_extension(sbi_rt::Hsm).is_unavailable() {
        warn!("HSM SBI extension is not supported for current SEE.");
        return;
    }
    sbi_rt::hart_start(hartid, entry.as_usize(), stack_top.as_usize());
}

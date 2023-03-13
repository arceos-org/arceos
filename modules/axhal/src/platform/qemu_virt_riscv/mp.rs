use crate::mem::PhysAddr;

pub fn start_secondary_cpu(hardid: usize, entry: PhysAddr, stack_top: PhysAddr) {
    if sbi_rt::probe_extension(sbi_rt::Hsm).is_unavailable() {
        warn!("HSM SBI extension is not supported for current SEE.");
        return;
    }
    sbi_rt::hart_start(hardid, entry.as_usize(), stack_top.as_usize());
}

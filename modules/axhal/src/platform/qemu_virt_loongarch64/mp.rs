use crate::mem::{phys_to_virt, PhysAddr};
use loongarch64::ipi::{csr_mail_send, send_ipi_single};

/// Starts the given secondary CPU with its boot stack.
pub fn start_secondary_cpu(hartid: usize, stack_top: PhysAddr) {
    extern "C" {
        fn _start_secondary();
    }
    let stack_top_virt_addr = phys_to_virt(stack_top).as_usize();
    unsafe {
        super::boot::SMP_BOOT_STACK_TOP = stack_top_virt_addr;
    }
    csr_mail_send(_start_secondary as u64, hartid, 0);
    send_ipi_single(hartid, 1);
}

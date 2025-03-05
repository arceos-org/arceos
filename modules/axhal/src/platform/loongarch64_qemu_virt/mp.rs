use loongArch64::ipi::{csr_mail_send, send_ipi_single};

use crate::mem::phys_to_virt;

const ACTION_BOOT_CPU: u32 = 1;

pub static mut SMP_BOOT_STACK_TOP: usize = 0;

/// Starts the given secondary CPU with its boot stack.
pub fn start_secondary_cpu(cpu_id: usize, stack_top: crate::mem::PhysAddr) {
    unsafe extern "C" {
        fn _start_secondary();
    }
    let stack_top_virt_addr = phys_to_virt(stack_top).as_usize();
    unsafe {
        SMP_BOOT_STACK_TOP = stack_top_virt_addr;
    }
    csr_mail_send(_start_secondary as usize as _, cpu_id, 0);
    send_ipi_single(cpu_id, ACTION_BOOT_CPU);
}

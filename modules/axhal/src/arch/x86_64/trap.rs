#[cfg(feature = "monolithic")]
use crate::trap::handle_page_fault;

#[cfg(feature = "monolithic")]
use page_table_entry::MappingFlags;

use x86::{controlregs::cr2, irq::*};

use super::context::TrapFrame;

core::arch::global_asm!(include_str!("trap.S"));

const IRQ_VECTOR_START: u8 = 0x20;
const IRQ_VECTOR_END: u8 = 0xff;

#[no_mangle]
fn x86_trap_handler(tf: &mut TrapFrame) {
    match tf.vector as u8 {
        PAGE_FAULT_VECTOR => {
            if tf.is_user() {
                warn!(
                    "User #PF @ {:#x}, fault_vaddr={:#x}, error_code={:#x}",
                    tf.rip,
                    unsafe { cr2() },
                    tf.error_code,
                );
                #[cfg(feature = "monolithic")]
                {
                    //  31              15                             4               0
                    // +---+--  --+---+-----+---+--  --+---+----+----+---+---+---+---+---+
                    // |   Reserved   | SGX |   Reserved   | SS | PK | I | R | U | W | P |
                    // +---+--  --+---+-----+---+--  --+---+----+----+---+---+---+---+---+
                    let mut map_flags = MappingFlags::USER; // TODO: add this flags through user tf.
                    if tf.error_code & (1 << 1) != 0 {
                        map_flags |= MappingFlags::WRITE;
                    }
                    if tf.error_code & (1 << 2) != 0 {
                        map_flags |= MappingFlags::USER;
                    }
                    if tf.error_code & (1 << 3) != 0 {
                        map_flags |= MappingFlags::READ;
                    }
                    if tf.error_code & (1 << 4) != 0 {
                        map_flags |= MappingFlags::EXECUTE;
                    }
                    axlog::debug!("error_code: {:?}", tf.error_code);
                    handle_page_fault(unsafe { cr2() }.into(), map_flags);
                }
            } else {
                panic!(
                    "Kernel #PF @ {:#x}, fault_vaddr={:#x}, error_code={:#x}:\n{:#x?}",
                    tf.rip,
                    unsafe { cr2() },
                    tf.error_code,
                    tf,
                );
            }
        }
        BREAKPOINT_VECTOR => debug!("#BP @ {:#x} ", tf.rip),
        GENERAL_PROTECTION_FAULT_VECTOR => {
            panic!(
                "#GP @ {:#x}, error_code={:#x}:\n{:#x?}",
                tf.rip, tf.error_code, tf
            );
        }
        IRQ_VECTOR_START..=IRQ_VECTOR_END => crate::trap::handle_irq_extern(tf.vector as _, false),
        _ => {
            panic!(
                "Unhandled exception {} (error_code = {:#x}) @ {:#x}:\n{:#x?}",
                tf.vector, tf.error_code, tf.rip, tf
            );
        }
    }
    #[cfg(feature = "signal")]
    if tf.is_user() {
        crate::trap::handle_signal();
    }
}

#[no_mangle]
#[cfg(feature = "monolithic")]
/// To handle the first time into the user space
///
/// 1. push the given trap frame into the kernel stack
/// 2. go into the user space
///
/// args:
///
/// 1. kernel_sp: the top of the kernel stack
///
/// 2. frame_base: the address of the trap frame which will be pushed into the kernel stack
pub fn first_into_user(kernel_sp: usize, frame_base: usize) {
    // Make sure that all csr registers are stored before enable the interrupt
    use memory_addr::VirtAddr;

    use crate::arch::flush_tlb;

    use super::disable_irqs;
    disable_irqs();
    flush_tlb(None);

    let trap_frame_size = core::mem::size_of::<TrapFrame>();
    let kernel_base = kernel_sp - trap_frame_size;
    crate::set_tss_stack_top(VirtAddr::from(kernel_sp));
    unsafe {
        *(kernel_base as *mut TrapFrame) = *(frame_base as *const TrapFrame);
        core::arch::asm!(
            r"
                    mov     gs:[offset __PERCPU_KERNEL_RSP_OFFSET], {kernel_sp}

                    mov      rsp, {kernel_base}

                    pop rax
                    pop rcx
                    pop rdx
                    pop rbx
                    pop rbp
                    pop rsi
                    pop rdi
                    pop r8
                    pop r9
                    pop r10
                    pop r11
                    pop r12
                    pop r13
                    pop r14
                    pop r15
                    add rsp, 16

                    swapgs
                    iretq
                ",
            kernel_sp = in(reg) kernel_sp,
            kernel_base = in(reg) kernel_base,
        );
    };
}

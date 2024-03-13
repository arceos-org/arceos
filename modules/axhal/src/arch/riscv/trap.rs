#[cfg(feature = "monolithic")]
use page_table_entry::MappingFlags;

#[cfg(feature = "monolithic")]
use riscv::register::{sepc, stval};

use riscv::register::scause::{self, Exception as E, Trap};

#[cfg(feature = "monolithic")]
use crate::trap::handle_page_fault;

#[cfg(feature = "signal")]
use crate::trap::handle_signal;

#[allow(unused)]
use super::{disable_irqs, TrapFrame};

#[cfg(feature = "monolithic")]
use super::enable_irqs;

#[cfg(feature = "monolithic")]
use crate::trap::handle_syscall;

include_asm_marcos!();

core::arch::global_asm!(
    include_str!("trap.S"),
    trapframe_size = const core::mem::size_of::<TrapFrame>(),
);

fn handle_breakpoint(sepc: &mut usize) {
    debug!("Exception(Breakpoint) @ {:#x} ", sepc);
    *sepc += 2
}

#[no_mangle]
#[allow(unused)]
fn riscv_trap_handler(tf: &mut TrapFrame, from_user: bool) {
    let scause = scause::read();
    #[cfg(feature = "monolithic")]
    axfs_ramfs::INTERRUPT.lock().record(scause.code());
    match scause.cause() {
        Trap::Exception(E::Breakpoint) => handle_breakpoint(&mut tf.sepc),
        Trap::Interrupt(_) => crate::trap::handle_irq_extern(scause.bits(), from_user),
        #[cfg(feature = "monolithic")]
        Trap::Exception(E::UserEnvCall) => {
            enable_irqs();
            // jump to next instruction anyway
            tf.sepc += 4;
            // get system call return value
            let result = handle_syscall(
                tf.regs.a7,
                [
                    tf.regs.a0, tf.regs.a1, tf.regs.a2, tf.regs.a3, tf.regs.a4, tf.regs.a5,
                ],
            );
            // cx is changed during sys_exec, so we have to call it again
            tf.regs.a0 = result as usize;
        }
        #[cfg(feature = "monolithic")]
        Trap::Exception(E::InstructionPageFault) => {
            let addr = stval::read();
            if !from_user {
                unimplemented!(
                    "I page fault from kernel, addr: {:X}, sepc: {:X}",
                    addr,
                    tf.sepc
                );
            }
            handle_page_fault(addr.into(), MappingFlags::USER | MappingFlags::EXECUTE);
        }

        #[cfg(feature = "monolithic")]
        Trap::Exception(E::LoadPageFault) => {
            let addr = stval::read();
            if !from_user {
                error!("L page fault from kernel, addr: {:#x}", addr);
                unimplemented!("L page fault from kernel");
            }
            handle_page_fault(addr.into(), MappingFlags::USER | MappingFlags::READ);
        }

        #[cfg(feature = "monolithic")]
        Trap::Exception(E::StorePageFault) => {
            if !from_user {
                error!(
                    "S page fault from kernel, addr: {:#x} sepc:{:X}",
                    stval::read(),
                    sepc::read()
                );
                unimplemented!("S page fault from kernel");
            }
            let addr = stval::read();
            handle_page_fault(addr.into(), MappingFlags::USER | MappingFlags::WRITE);
        }

        _ => {
            panic!(
                "Unhandled trap {:?} @ {:#x}:\n{:#x?}",
                scause.cause(),
                tf.sepc,
                tf
            );
        }
    }

    #[cfg(feature = "signal")]
    if from_user {
        handle_signal();
    }

    #[cfg(feature = "monolithic")]
    // 在保证将寄存器都存储好之后，再开启中断
    // 否则此时会因为写入csr寄存器过程中出现中断，导致出现异常
    disable_irqs();
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
    use crate::arch::flush_tlb;

    disable_irqs();
    flush_tlb(None);

    let trap_frame_size = core::mem::size_of::<TrapFrame>();
    let kernel_base = kernel_sp - trap_frame_size;
    unsafe {
        core::arch::asm!(
            r"
            mv      sp, {frame_base}
            .short  0x2432                      // fld fs0,264(sp)
            .short  0x24d2                      // fld fs1,272(sp)
            mv      t1, {kernel_base}
            LDR     t0, sp, 2
            STR     gp, t1, 2
            mv      gp, t0
            LDR     t0, sp, 3
            STR     tp, t1, 3                   // save supervisor tp. Note that it is stored on the kernel stack rather than in sp, in which case the ID of the currently running CPU should be stored
            mv      tp, t0                      // tp: now it stores the TLS pointer to the corresponding thread
            csrw    sscratch, {kernel_sp}       // put supervisor sp to scratch
            LDR     t0, sp, 31
            LDR     t1, sp, 32
            csrw    sepc, t0
            csrw    sstatus, t1
            POP_GENERAL_REGS
            LDR     sp, sp, 1
            sret
        ",
            frame_base = in(reg) frame_base,
            kernel_sp = in(reg) kernel_sp,
            kernel_base = in(reg) kernel_base,
        );
    };
}

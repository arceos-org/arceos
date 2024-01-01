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
        Trap::Interrupt(_) => crate::trap::handle_irq_extern(scause.bits()),
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
            handle_page_fault(addr.into(), MappingFlags::USER | MappingFlags::EXECUTE, tf);
        }

        #[cfg(feature = "monolithic")]
        Trap::Exception(E::LoadPageFault) => {
            let addr = stval::read();
            if !from_user {
                error!("L page fault from kernel, addr: {:#x}", addr);
                unimplemented!("L page fault from kernel");
            }
            handle_page_fault(addr.into(), MappingFlags::USER | MappingFlags::READ, tf);
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
            handle_page_fault(addr.into(), MappingFlags::USER | MappingFlags::WRITE, tf);
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

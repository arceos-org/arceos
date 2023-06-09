use page_table::MappingFlags;
use riscv::register::{
    scause::{self, Exception as E, Trap},
    stval,
};

#[cfg(all(feature = "paging", feature = "monolithic"))]
use crate::trap::{handle_page_fault, handle_syscall};

use super::TrapFrame;

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
fn riscv_trap_handler(tf: &mut TrapFrame, from_user: bool) {
    let scause = scause::read();
    match scause.cause() {
        Trap::Exception(E::Breakpoint) => handle_breakpoint(&mut tf.sepc),
        Trap::Interrupt(_) => crate::trap::handle_irq_extern(scause.bits()),
        #[cfg(feature = "monolithic")]
        Trap::Exception(E::UserEnvCall) => {
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

        #[cfg(all(feature = "paging", feature = "monolithic"))]
        Trap::Exception(E::InstructionPageFault) => {
            if !from_user {
                unimplemented!("I page fault from kernel");
            }

            let addr = stval::read();

            handle_page_fault(addr.into(), MappingFlags::USER | MappingFlags::EXECUTE);
        }

        #[cfg(all(feature = "paging", feature = "monolithic"))]
        Trap::Exception(E::LoadPageFault) => {
            if !from_user {
                unimplemented!("L page fault from kernel");
            }
            let addr = stval::read();

            handle_page_fault(addr.into(), MappingFlags::USER | MappingFlags::READ);
        }

        #[cfg(all(feature = "paging", feature = "monolithic"))]
        Trap::Exception(E::StorePageFault) => {
            if !from_user {
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
}

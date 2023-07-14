use page_table::MappingFlags;
use riscv::register::{
    scause::{self, Exception as E, Trap},
    sepc, stval,
};

#[cfg(feature = "paging")]
use crate::trap::handle_page_fault;

#[cfg(feature = "signal")]
use crate::trap::handle_signal;

#[cfg(feature = "monolithic")]
use crate::trap::handle_syscall;

use super::{disable_irqs, enable_irqs, TrapFrame};

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
fn riscv_trap_handler(tf: &mut TrapFrame, mut from_user: bool) {
    let scause = scause::read();
    if (tf.sepc as isize) < 0 {
        from_user = false;
    }
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

        #[cfg(feature = "paging")]
        Trap::Exception(E::InstructionPageFault) => {
            if !from_user {
                unimplemented!("I page fault from kernel");
            }

            let addr = stval::read();

            handle_page_fault(addr.into(), MappingFlags::USER | MappingFlags::EXECUTE);
        }

        #[cfg(feature = "paging")]
        Trap::Exception(E::LoadPageFault) => {
            let addr = stval::read();
            if !from_user {
                info!("L page fault from kernel, addr: {:#x}", addr);
                unimplemented!("L page fault from kernel");
            }
            handle_page_fault(addr.into(), MappingFlags::USER | MappingFlags::READ);
        }

        #[cfg(feature = "paging")]
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
                "Unhandled trap {:?} @ {:#x}:\n{:#x?} from_user: {}",
                scause.cause(),
                tf.sepc,
                tf,
                from_user
            );
        }
    }
    #[cfg(feature = "signal")]
    if !(!from_user && scause.cause() == Trap::Interrupt(scause::Interrupt::SupervisorTimer)) {
        handle_signal();
    }
    // 在保证将寄存器都存储好之后，再开启中断
    // 否则此时会因为写入csr寄存器过程中出现中断，导致出现异常
    disable_irqs();
}

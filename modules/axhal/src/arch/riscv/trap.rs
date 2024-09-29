use page_table_entry::MappingFlags;
use riscv::register::scause::{self, Exception as E, Trap};
use riscv::register::stval;

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

fn handle_page_fault(tf: &TrapFrame, mut access_flags: MappingFlags, is_user: bool) {
    if is_user {
        access_flags |= MappingFlags::USER;
    }
    let vaddr = va!(stval::read());
    if !handle_trap!(PAGE_FAULT, vaddr, access_flags, is_user) {
        panic!(
            "Unhandled {} Page Fault @ {:#x}, fault_vaddr={:#x} ({:?}):\n{:#x?}",
            if is_user { "User" } else { "Supervisor" },
            tf.sepc,
            vaddr,
            access_flags,
            tf,
        );
    }
}

#[no_mangle]
fn riscv_trap_handler(tf: &mut TrapFrame, from_user: bool) {
    let scause = scause::read();
    match scause.cause() {
        #[cfg(feature = "uspace")]
        Trap::Exception(E::UserEnvCall) => {
            tf.regs.a0 = crate::trap::handle_syscall(tf, tf.regs.a7) as usize;
            tf.sepc += 4;
        }
        Trap::Exception(E::LoadPageFault) => handle_page_fault(tf, MappingFlags::READ, from_user),
        Trap::Exception(E::StorePageFault) => handle_page_fault(tf, MappingFlags::WRITE, from_user),
        Trap::Exception(E::InstructionPageFault) => {
            handle_page_fault(tf, MappingFlags::EXECUTE, from_user)
        }
        Trap::Exception(E::Breakpoint) => handle_breakpoint(&mut tf.sepc),
        Trap::Interrupt(_) => {
            handle_trap!(IRQ, scause.bits());
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

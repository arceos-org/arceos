use memory_addr::VirtAddr;
use page_table_entry::MappingFlags;
use riscv::interrupt::Trap;
use riscv::interrupt::supervisor::{Exception as E, Interrupt as I};
use riscv::register::{scause, stval};

use super::TrapFrame;

core::arch::global_asm!(
    include_asm_macros!(),
    include_str!("trap.S"),
    trapframe_size = const core::mem::size_of::<TrapFrame>(),
);

fn handle_breakpoint(sepc: &mut usize) {
    debug!("Exception(Breakpoint) @ {:#x} ", sepc);
    *sepc += 2
}

fn handle_page_fault(
    tf: &TrapFrame,
    vaddr: VirtAddr,
    mut access_flags: MappingFlags,
    is_user: bool,
) {
    if is_user {
        access_flags |= MappingFlags::USER;
    }
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

#[unsafe(no_mangle)]
fn riscv_trap_handler(tf: &mut TrapFrame, from_user: bool) {
    let scause = scause::read();
    if let Ok(cause) = scause.cause().try_into::<I, E>() {
        // Interrupts modify the value of `stval`, which must be saved before the
        // interrupt is enabled
        let vaddr = va!(stval::read());
        if scause.is_exception() {
            unmask_irqs(tf);
        }
        match cause {
            #[cfg(feature = "uspace")]
            Trap::Exception(E::UserEnvCall) => {
                tf.sepc += 4;
                tf.regs.a0 = crate::trap::handle_syscall(tf, tf.regs.a7) as usize;
            }
            Trap::Exception(E::LoadPageFault) => {
                handle_page_fault(tf, vaddr, MappingFlags::READ, from_user)
            }
            Trap::Exception(E::StorePageFault) => {
                handle_page_fault(tf, vaddr, MappingFlags::WRITE, from_user)
            }
            Trap::Exception(E::InstructionPageFault) => {
                handle_page_fault(tf, vaddr, MappingFlags::EXECUTE, from_user)
            }
            Trap::Exception(E::Breakpoint) => handle_breakpoint(&mut tf.sepc),
            Trap::Interrupt(_) => {
                handle_trap!(IRQ, scause.bits());
            }
            _ => {
                panic!("Unhandled trap {:?} @ {:#x}:\n{:#x?}", cause, tf.sepc, tf);
            }
        }
        crate::trap::post_trap_callback(tf, from_user);
        mask_irqs();
    } else {
        panic!(
            "Unknown trap {:?} @ {:#x}:\n{:#x?}",
            scause.cause(),
            tf.sepc,
            tf
        );
    }
}

// Interrupt unmasking function for exception handling.
// NOTE: It must be invoked after the switch to kernel mode has finished
//
// If interrupts were enabled before the exception (the `SPIE` bit in the
// `sstatus` register is set), re-enable interrupts before exception handling
//
// On riscv64, when an exception occurs, `sstatus.SIE` is set to zero to mask
// the interrupt and the old value of `SIE` is stored in SPIE. Recover `SIE`
// according to `SPIE` when using `sret`.
fn unmask_irqs(tf: &TrapFrame) {
    const PIE: usize = 1 << 5;
    if tf.sstatus & PIE == PIE {
        super::enable_irqs();
    } else {
        debug!("Interrupts were disabled before exception");
    }
}

fn mask_irqs() {
    super::disable_irqs();
}

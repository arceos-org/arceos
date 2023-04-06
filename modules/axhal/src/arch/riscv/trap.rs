use riscv::register::scause::{self, Exception as E, Trap};

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
fn riscv_trap_handler(tf: &mut TrapFrame, _from_user: bool) {
    let scause = scause::read();
    match scause.cause() {
        Trap::Exception(E::Breakpoint) => handle_breakpoint(&mut tf.sepc),
        Trap::Interrupt(_) => crate::trap::handle_irq_extern(scause.bits()),
        
        #[cfg(feature = "user")]
        Trap::Exception(E::UserEnvCall) => {
            tf.sepc += 4;
            let ret = crate::trap::handle_syscall_extern(
                tf.regs.a7,
                [
                    tf.regs.a0, tf.regs.a1,
                    tf.regs.a2, tf.regs.a3,
                    tf.regs.a4, tf.regs.a5,
                ]
            );
            tf.regs.a0 = ret as usize;
        },
        
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

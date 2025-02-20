use super::context::TrapFrame;
use loongArch64::register::{
    badv,
    estat::{self, Exception, Trap},
};
use page_table_entry::MappingFlags;

core::arch::global_asm!(
    include_str!("trap.S"),
    trapframe_size = const (core::mem::size_of::<TrapFrame>()),
);

fn handle_breakpoint(era: &mut usize) {
    debug!("Exception(Breakpoint) @ {:#x} ", era);
    *era += 4;
}

fn handle_page_fault(tf: &TrapFrame, mut access_flags: MappingFlags, is_user: bool) {
    if is_user {
        access_flags |= MappingFlags::USER;
    }
    let vaddr = va!(badv::read().raw());
    if !handle_trap!(PAGE_FAULT, vaddr, access_flags, is_user) {
        panic!(
            "Unhandled {} Page Fault @ {:#x}, fault_vaddr={:#x} ({:?}):\n{:#x?}",
            if is_user { "User" } else { "Supervisor" },
            tf.badv,
            vaddr,
            access_flags,
            tf,
        );
    }
}

#[unsafe(no_mangle)]
fn loongarch64_trap_handler(tf: &mut TrapFrame, from_user: bool) {
    let estat = estat::read();

    match estat.cause() {
        #[cfg(feature = "uspace")]
        Trap::Exception(Exception::Syscall) => {
            tf.regs[4] = crate::trap::handle_syscall(tf, tf.regs[11]) as usize;
            tf.era += 4;
        }
        Trap::Exception(Exception::LoadPageFault) | Trap::Exception(Exception::FetchPageFault) => {
            handle_page_fault(tf, MappingFlags::READ, from_user)
        }
        Trap::Exception(Exception::StorePageFault) => {
            handle_page_fault(tf, MappingFlags::WRITE, from_user)
        }
        Trap::Exception(Exception::Breakpoint) => handle_breakpoint(&mut tf.era),
        Trap::Interrupt(_) => {
            let irq_num: usize = estat.is().trailing_zeros() as usize;
            handle_trap!(IRQ, irq_num);
        }
        _ => {
            panic!(
                "Unhandled trap {:?} @ {:#x}:\n{:#x?}",
                estat.cause(),
                tf.era,
                tf
            );
        }
    }
}

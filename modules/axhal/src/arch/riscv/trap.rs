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
#[cfg(feature = "monolithic")]
/// 手动进入用户态
///
/// 1. 将对应trap上下文压入内核栈
/// 2. 返回用户态
///
/// args：
///
/// 1. kernel_sp：内核栈顶
///
/// 2. frame_base：对应即将压入内核栈的trap上下文的地址
pub fn first_into_user(kernel_sp: usize, frame_base: usize) -> ! {
    use crate::arch::disable_irqs;

    let trap_frame_size = core::mem::size_of::<TrapFrame>();
    let kernel_base = kernel_sp - trap_frame_size;
    // 在保证将寄存器都存储好之后，再开启中断
    // 否则此时会因为写入csr寄存器过程中出现中断，导致出现异常
    disable_irqs();
    // 在内核态中，tp寄存器存储的是当前任务的CPU ID
    // 而当从内核态进入到用户态时，会将tp寄存器的值先存储在内核栈上，即把该任务对应的CPU ID存储在内核栈上
    // 然后将tp寄存器的值改为对应线程的tls指针的值
    // 因此在用户态中，tp寄存器存储的值是线程的tls指针的值
    // 而当从用户态进入到内核态时，会先将内核栈上的值读取到某一个中间寄存器t0中，然后将tp的值存入内核栈
    // 然后再将t0的值赋给tp，因此此时tp的值是当前任务的CPU ID
    // 对应实现在axhal/src/arch/riscv/trap.S中
    unsafe {
        riscv::asm::sfence_vma_all();
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
            STR     tp, t1, 3                   // save supervisor tp，注意是存储到内核栈上而不是sp中，此时存储的应该是当前运行的CPU的ID
            mv      tp, t0                      // tp：本来存储的是CPU ID，在这个时候变成了对应线程的TLS 指针
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
    core::panic!("already in user mode!")
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

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
fn riscv_trap_handler(tf: &mut TrapFrame, from_user: bool) {
    #[cfg(feature = "user-paging")]
    {
        extern "C" {
            fn trap_vector_base();
        }
        unsafe {
            riscv::register::stvec::write(
                trap_vector_base as usize,
                riscv::register::stvec::TrapMode::Direct
            )
        }
    }
    
    #[cfg(feature = "user-paging")]
    let tf: &mut TrapFrame = unsafe {
        &mut *(crate::trap::get_current_trap_frame())
    };
    
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
                "Unhandled trap {:?} (stval = {:x}) @ {:#x}:\n{:#x?}",
                scause.cause(),
                riscv::register::stval::read(),
                tf.sepc,
                tf,
            );
        }
    }

    #[cfg(feature = "user-paging")]
    if from_user {
        let tf = crate::trap::get_current_trap_frame_virt_addr();
        let satp = crate::trap::get_current_satp();
        enter_uspace(tf, satp);
    }
    
}

#[cfg(feature = "user-paging")]
pub fn first_uentry() -> ! {
    let tf = crate::trap::get_current_trap_frame_virt_addr();
    let satp = crate::trap::get_current_satp();
    enter_uspace(tf, satp);    
}

#[cfg(feature = "user-paging")]
pub fn enter_uspace(tf: usize, satp: usize) -> ! {
    extern "C" {
        fn strampoline();
    }
    let satp = (8usize << 60) | (satp >> 12);
    unsafe {
        riscv::register::stvec::write(
            trap_from_uspace as usize - strampoline as usize + 0xffff_ffc0_0000_0000,
            riscv::register::stvec::TrapMode::Direct
        );
        let entry = __enter_uspace as usize - strampoline as usize + 0xffff_ffc0_0000_0000;
        core::arch::asm!(r"
            fence.i
            jr {entry}
        ",
            in("a0") tf,
            in("a1") satp,
            entry = in(reg) entry,             
        );
    }
    unreachable!();
    
}


#[cfg(feature = "user-paging")]
#[no_mangle]
#[link_section = ".text.trampoline"]
#[naked]
pub fn __enter_uspace() -> !{
    // a0: _tf, a1: _satp
    unsafe {
        core::arch::asm!(r"
            csrw    satp, a1
            sfence.vma      // into user space
            
            mv      sp, a0            
            LDR     gp, sp, 2      // load user gp and tp
            LDR     t0, sp, 3
            STR     tp, sp, 3      // save supervisor tp
            mv      tp, t0

            LDR     t0, sp, 31
            LDR     t1, sp, 32
            csrw    sepc, t0
            csrw    sstatus, t1
            csrw    sscratch, a0
         
            POP_GENERAL_REGS            
            LDR     sp, sp, 1
            sret
        ",
            options(noreturn)
        )
    }
}
#[cfg(feature = "user-paging")]
#[no_mangle]
#[link_section = ".text.trampoline"]
#[naked]
pub fn trap_from_uspace() -> ! {
    unsafe {
        core::arch::asm!(r"
            csrrw   sp, sscratch, sp

            PUSH_GENERAL_REGS
            csrr    t0, sepc
            csrr    t1, sstatus
            csrrw   t2, sscratch, zero

            STR     t0, sp, 31     // tf.sepc
            STR     t1, sp, 32     // tf.sstatus
            STR     t2, sp, 1      // tf.regs.sp

            LDR     t0, sp, 3      // load supervisor tp
            STR     gp, sp, 2      // save user gp and tp
            STR     tp, sp, 3
            mv      tp, t0

            LDR     t0, sp, 34     // tf.satp
            LDR     t1, sp, 35     // tf.trap_handler
            LDR     sp, sp, 33     // tf.kstack
            csrw    satp, t0
            sfence.vma             // kernel space
            li      a1, 1

            jr      t1
        ",
            options(noreturn)
        )
    }
}

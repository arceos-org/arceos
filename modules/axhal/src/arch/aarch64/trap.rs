use core::arch::global_asm;

use aarch64_cpu::registers::{ESR_EL1, FAR_EL1, SP_EL1};
use tock_registers::interfaces::Readable;

use super::TrapFrame;

global_asm!(include_str!("trap.S"));

#[cfg(feature = "monolithic")]
use crate::arch::{disable_irqs, enable_irqs};

#[cfg(feature = "monolithic")]
use crate::trap::handle_syscall;

#[cfg(feature = "signal")]
use crate::trap::handle_signal;

#[repr(u8)]
#[derive(Debug)]
#[allow(dead_code)]
enum TrapKind {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[repr(u8)]
#[derive(Debug)]
#[allow(dead_code)]
enum TrapSource {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[allow(dead_code)]
extern "C" {
    fn ret_to_first_user(sp: usize);
}

#[no_mangle]
fn invalid_exception(tf: &TrapFrame, kind: TrapKind, source: TrapSource) {
    panic!(
        "Invalid exception {:?} from {:?}:\n{:#x?}",
        kind, source, tf
    );
}

#[no_mangle]
fn handle_el1t_64_sync_exception(tf: &mut TrapFrame) {
    invalid_exception(tf, TrapKind::Synchronous, TrapSource::CurrentSpEl0);
}

#[no_mangle]
fn handle_el1t_64_irq_exception(tf: &mut TrapFrame) {
    invalid_exception(tf, TrapKind::Irq, TrapSource::CurrentSpEl0);
}

#[no_mangle]
fn handle_el1t_64_fiq_exception(tf: &mut TrapFrame) {
    invalid_exception(tf, TrapKind::Fiq, TrapSource::CurrentSpEl0);
}

#[no_mangle]
fn handle_el1t_64_error_exception(tf: &mut TrapFrame) {
    invalid_exception(tf, TrapKind::SError, TrapSource::CurrentSpEl0);
}

#[no_mangle]
fn handle_el1h_64_sync_exception(tf: &mut TrapFrame) {
    let esr = ESR_EL1.extract();

    match esr.read_as_enum(ESR_EL1::EC) {
        Some(ESR_EL1::EC::Value::Brk64) => {
            let iss = esr.read(ESR_EL1::ISS);
            debug!("BRK #{:#x} @ {:#x} ", iss, tf.elr);
            tf.elr += 4;
        }
        Some(ESR_EL1::EC::Value::DataAbortCurrentEL)
        | Some(ESR_EL1::EC::Value::InstrAbortCurrentEL) => {
            let iss = esr.read(ESR_EL1::ISS);
            panic!(
                "EL1 Page Fault @ {:#x}, FAR={:#x}, ISS={:#x}:\n{:#x?}",
                tf.elr,
                FAR_EL1.get(),
                iss,
                tf,
            );
        }
        _ => {
            panic!(
                "Unhandled synchronous exception @ {:#x}: ESR={:#x} (EC {:#08b}, ISS {:#x}) SP{:#x}",
                tf.elr,
                esr.get(),
                esr.read(ESR_EL1::EC),
                esr.read(ESR_EL1::ISS),
                SP_EL1.get(),
            );
        }
    }
}

#[no_mangle]
fn handle_el1h_64_irq_exception(_tf: &TrapFrame) {
    crate::trap::handle_irq_extern(0, false);
}

#[no_mangle]
fn handle_el1h_64_fiq_exception(tf: &mut TrapFrame) {
    invalid_exception(tf, TrapKind::Fiq, TrapSource::CurrentSpElx);
}

#[no_mangle]
fn handle_el1h_64_error_exception(tf: &mut TrapFrame) {
    invalid_exception(tf, TrapKind::SError, TrapSource::CurrentSpElx);
}

#[no_mangle]
#[cfg(feature = "monolithic")]
fn handle_el0t_64_sync_exception(tf: &mut TrapFrame) {
    let esr = ESR_EL1.extract();

    match esr.read_as_enum(ESR_EL1::EC) {
        Some(ESR_EL1::EC::Value::SVC64) => {
            info!(
                "task: {:p} into svc {}",
                crate::cpu::current_task_ptr::<u8>(),
                tf.r[8]
            );
            enable_irqs();
            let result = handle_syscall(
                tf.r[8],
                [tf.r[0], tf.r[1], tf.r[2], tf.r[3], tf.r[4], tf.r[5]],
            );
            tf.r[0] = result as usize;
        }
        Some(ESR_EL1::EC::Value::DataAbortLowerEL) => {
            let far = FAR_EL1.get() as usize;
            enable_irqs();
            super::mem_fault::el0_da(far, esr.get(), tf);
        }
        Some(ESR_EL1::EC::Value::InstrAbortLowerEL) => {
            let far = FAR_EL1.get() as usize;
            enable_irqs();
            info!("data abort page fault at addr {:#x?}", far);
            super::mem_fault::el0_ia(far, esr.get(), tf);
        }
        _ => {
            panic!(
                "Unhandled synchronous exception @ {:#x}: ESR={:#x} (EC {:#08b}, ISS {:#x})",
                tf.elr,
                esr.get(),
                esr.read(ESR_EL1::EC),
                esr.read(ESR_EL1::ISS),
            );
        }
    }

    handle_signal();

    disable_irqs();
}

#[no_mangle]
#[cfg(not(feature = "monolithic"))]
fn handle_el0t_64_sync_exception(tf: &mut TrapFrame) {
    invalid_exception(tf, TrapKind::Synchronous, TrapSource::LowerAArch64);
}

#[no_mangle]
#[cfg(feature = "monolithic")]
fn handle_el0t_64_irq_exception(_tf: &TrapFrame) {
    crate::trap::handle_irq_extern(0, true);
    handle_signal();
}

#[no_mangle]
#[cfg(not(feature = "monolithic"))]
fn handle_el0t_64_irq_exception(tf: &TrapFrame) {
    invalid_exception(tf, TrapKind::Irq, TrapSource::LowerAArch64);
}

#[no_mangle]
fn handle_el0t_64_fiq_exception(tf: &TrapFrame) {
    invalid_exception(tf, TrapKind::Fiq, TrapSource::LowerAArch64);
}

#[no_mangle]
fn handle_el0t_64_error_exception(tf: &TrapFrame) {
    invalid_exception(tf, TrapKind::SError, TrapSource::LowerAArch64);
}

#[no_mangle]
fn handle_el0t_32_sync_exception(tf: &TrapFrame) {
    invalid_exception(tf, TrapKind::Synchronous, TrapSource::LowerAArch32);
}

#[no_mangle]
fn handle_el0t_32_irq_exception(tf: &TrapFrame) {
    invalid_exception(tf, TrapKind::Irq, TrapSource::LowerAArch32);
}

#[no_mangle]
fn handle_el0t_32_fiq_exception(tf: &TrapFrame) {
    invalid_exception(tf, TrapKind::Fiq, TrapSource::LowerAArch32);
}

#[no_mangle]
fn handle_el0t_32_error_exception(tf: &TrapFrame) {
    invalid_exception(tf, TrapKind::SError, TrapSource::LowerAArch32);
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
pub fn first_into_user(kernel_sp: usize, frame_base: usize) -> ! {
    let trap_frame_size = core::mem::size_of::<TrapFrame>();
    let kernel_base = kernel_sp - trap_frame_size;
    info!("frame_base sp {:#x} kernel_sp{:#x}", frame_base, kernel_sp);
    // 在保证将寄存器都存储好之后，再开启中断
    disable_irqs();
    crate::arch::flush_tlb(None);
    crate::arch::flush_icache_all();
    //crate::arch::flush_dcache_all();
    assert_eq!(kernel_base, frame_base);
    unsafe {
        ret_to_first_user(kernel_base);
    };
    core::panic!("already in user mode!")
}

use core::arch::global_asm;

use aarch64_cpu::registers::{ESR_EL1, FAR_EL1};
use page_table_entry::MappingFlags;
use tock_registers::interfaces::Readable;

use super::TrapFrame;

global_asm!(
    include_str!("trap.S"),
    trapframe_size = const core::mem::size_of::<TrapFrame>(),
    cache_current_task_ptr = sym crate::cpu::cache_current_task_ptr,
);

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

#[unsafe(no_mangle)]
fn invalid_exception(tf: &TrapFrame, kind: TrapKind, source: TrapSource) {
    panic!(
        "Invalid exception {:?} from {:?}:\n{:#x?}",
        kind, source, tf
    );
}

#[unsafe(no_mangle)]
fn handle_irq_exception(_tf: &TrapFrame) {
    handle_trap!(IRQ, 0);
}

fn handle_instruction_abort(tf: &TrapFrame, iss: u64, is_user: bool) {
    let mut access_flags = MappingFlags::EXECUTE;
    if is_user {
        access_flags |= MappingFlags::USER;
    }
    let vaddr = va!(FAR_EL1.get() as usize);

    // Only handle Translation fault and Permission fault
    if !matches!(iss & 0b111100, 0b0100 | 0b1100) // IFSC or DFSC bits
        || !handle_trap!(PAGE_FAULT, vaddr, access_flags, is_user)
    {
        panic!(
            "Unhandled {} Instruction Abort @ {:#x}, fault_vaddr={:#x}, ISS={:#x} ({:?}):\n{:#x?}",
            if is_user { "EL0" } else { "EL1" },
            tf.elr,
            vaddr,
            iss,
            access_flags,
            tf,
        );
    }
}

fn handle_data_abort(tf: &TrapFrame, iss: u64, is_user: bool) {
    let wnr = (iss & (1 << 6)) != 0; // WnR: Write not Read
    let cm = (iss & (1 << 8)) != 0; // CM: Cache maintenance
    let mut access_flags = if wnr & !cm {
        MappingFlags::WRITE
    } else {
        MappingFlags::READ
    };
    if is_user {
        access_flags |= MappingFlags::USER;
    }
    let vaddr = va!(FAR_EL1.get() as usize);

    // Only handle Translation fault and Permission fault
    if !matches!(iss & 0b111100, 0b0100 | 0b1100) // IFSC or DFSC bits
        || !handle_trap!(PAGE_FAULT, vaddr, access_flags, is_user)
    {
        panic!(
            "Unhandled {} Data Abort @ {:#x}, fault_vaddr={:#x}, ISS=0b{:08b} ({:?}):\n{:#x?}",
            if is_user { "EL0" } else { "EL1" },
            tf.elr,
            vaddr,
            iss,
            access_flags,
            tf,
        );
    }
}

#[unsafe(no_mangle)]
fn handle_sync_exception(tf: &mut TrapFrame) {
    let esr = ESR_EL1.extract();
    let iss = esr.read(ESR_EL1::ISS);
    match esr.read_as_enum(ESR_EL1::EC) {
        #[cfg(feature = "uspace")]
        Some(ESR_EL1::EC::Value::SVC64) => {
            tf.r[0] = crate::trap::handle_syscall(tf, tf.r[8] as usize) as u64;
        }
        Some(ESR_EL1::EC::Value::InstrAbortLowerEL) => handle_instruction_abort(tf, iss, true),
        Some(ESR_EL1::EC::Value::InstrAbortCurrentEL) => handle_instruction_abort(tf, iss, false),
        Some(ESR_EL1::EC::Value::DataAbortLowerEL) => handle_data_abort(tf, iss, true),
        Some(ESR_EL1::EC::Value::DataAbortCurrentEL) => handle_data_abort(tf, iss, false),
        Some(ESR_EL1::EC::Value::Brk64) => {
            debug!("BRK #{:#x} @ {:#x} ", iss, tf.elr);
            tf.elr += 4;
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
}

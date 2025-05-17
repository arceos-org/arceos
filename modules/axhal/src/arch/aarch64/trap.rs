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
impl TrapSource {
    fn is_from_user(&self) -> bool {
        matches!(self, TrapSource::LowerAArch64 | TrapSource::LowerAArch32)
    }
}

#[unsafe(no_mangle)]
fn invalid_exception(tf: &TrapFrame, kind: TrapKind, source: TrapSource) {
    panic!(
        "Invalid exception {:?} from {:?}:\n{:#x?}",
        kind, source, tf
    );
}

#[unsafe(no_mangle)]
fn handle_irq_exception(tf: &mut TrapFrame, source: TrapSource) {
    handle_trap!(IRQ, 0);
    crate::trap::post_trap_callback(tf, source.is_from_user());
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
fn handle_sync_exception(tf: &mut TrapFrame, source: TrapSource) {
    let esr = ESR_EL1.extract();
    let iss = esr.read(ESR_EL1::ISS);

    unmask_irqs(tf);

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
    crate::trap::post_trap_callback(tf, source.is_from_user());
    mask_irqs();
}

// Interrupt unmasking function for exception handling.
// NOTE: It must be invoked after the switch to kernel mode has finished
//
// If interrupts were enabled before the exception (`I` bit in `SPSR` is unmask),
// re-enable interrupts before handling the exception.
//
// On aarch64, when an exception occurs, the `CPSR` register value is stored in
// `SPSR_EL1`, where the `I` bit records whether the interrupt is enabled or not.
// `I::unmask` enable_irqs
fn unmask_irqs(tf: &TrapFrame) {
    const I_MASK: u64 = 1 << 7;
    if tf.spsr & I_MASK != I_MASK {
        super::enable_irqs();
    } else {
        debug!("Interrupts were disabled before exception");
    }
}

fn mask_irqs() {
    super::disable_irqs();
}

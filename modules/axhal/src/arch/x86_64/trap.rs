use page_table_entry::MappingFlags;
use x86::{controlregs::cr2, irq::*};
use x86_64::structures::idt::PageFaultErrorCode;

use super::context::TrapFrame;

core::arch::global_asm!(include_str!("trap.S"));

#[cfg(feature = "uspace")]
const LEGACY_SYSCALL_VECTOR: u8 = 0x80;

const IRQ_VECTOR_START: u8 = 0x20;
const IRQ_VECTOR_END: u8 = 0xff;

fn handle_page_fault(tf: &TrapFrame) {
    let access_flags = err_code_to_flags(tf.error_code)
        .unwrap_or_else(|e| panic!("Invalid #PF error code: {:#x}", e));
    let vaddr = va!(unsafe { cr2() });
    if !handle_trap!(PAGE_FAULT, vaddr, access_flags, tf.is_user()) {
        panic!(
            "Unhandled {} #PF @ {:#x}, fault_vaddr={:#x}, error_code={:#x} ({:?}):\n{:#x?}",
            if tf.is_user() { "user" } else { "kernel" },
            tf.rip,
            vaddr,
            tf.error_code,
            access_flags,
            tf,
        );
    }
}

#[unsafe(no_mangle)]
fn x86_trap_handler(tf: &mut TrapFrame) {
    #[cfg(feature = "uspace")]
    super::tls::switch_to_kernel_fs_base(tf);
    match tf.vector as u8 {
        PAGE_FAULT_VECTOR => handle_page_fault(tf),
        BREAKPOINT_VECTOR => debug!("#BP @ {:#x} ", tf.rip),
        GENERAL_PROTECTION_FAULT_VECTOR => {
            panic!(
                "#GP @ {:#x}, error_code={:#x}:\n{:#x?}",
                tf.rip, tf.error_code, tf
            );
        }
        #[cfg(feature = "uspace")]
        LEGACY_SYSCALL_VECTOR => super::syscall::handle_syscall(tf),
        IRQ_VECTOR_START..=IRQ_VECTOR_END => {
            handle_trap!(IRQ, tf.vector as _);
        }
        _ => {
            panic!(
                "Unhandled exception {} ({}, error_code={:#x}) @ {:#x}:\n{:#x?}",
                tf.vector,
                vec_to_str(tf.vector),
                tf.error_code,
                tf.rip,
                tf
            );
        }
    }
    #[cfg(feature = "uspace")]
    super::tls::switch_to_user_fs_base(tf);
}

fn vec_to_str(vec: u64) -> &'static str {
    if vec < 32 {
        EXCEPTIONS[vec as usize].mnemonic
    } else {
        "Unknown"
    }
}

fn err_code_to_flags(err_code: u64) -> Result<MappingFlags, u64> {
    let code = PageFaultErrorCode::from_bits_truncate(err_code);
    let reserved_bits = (PageFaultErrorCode::CAUSED_BY_WRITE
        | PageFaultErrorCode::USER_MODE
        | PageFaultErrorCode::INSTRUCTION_FETCH)
        .complement();
    if code.intersects(reserved_bits) {
        Err(err_code)
    } else {
        let mut flags = MappingFlags::empty();
        if code.contains(PageFaultErrorCode::CAUSED_BY_WRITE) {
            flags |= MappingFlags::WRITE;
        } else {
            flags |= MappingFlags::READ;
        }
        if code.contains(PageFaultErrorCode::USER_MODE) {
            flags |= MappingFlags::USER;
        }
        if code.contains(PageFaultErrorCode::INSTRUCTION_FETCH) {
            flags |= MappingFlags::EXECUTE;
        }
        Ok(flags)
    }
}

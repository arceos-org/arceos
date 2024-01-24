#[cfg(feature = "monolithic")]
use crate::trap::handle_page_fault;

#[cfg(feature = "monolithic")]
use page_table_entry::MappingFlags;

use x86::{controlregs::cr2, irq::*};

use super::context::TrapFrame;

core::arch::global_asm!(include_str!("trap.S"));

const IRQ_VECTOR_START: u8 = 0x20;
const IRQ_VECTOR_END: u8 = 0xff;

#[no_mangle]
fn x86_trap_handler(tf: &mut TrapFrame) {
    match tf.vector as u8 {
        PAGE_FAULT_VECTOR => {
            if tf.is_user() {
                warn!(
                    "User #PF @ {:#x}, fault_vaddr={:#x}, error_code={:#x}",
                    tf.rip,
                    unsafe { cr2() },
                    tf.error_code,
                );
                #[cfg(feature = "monolithic")]
                {
                    //  31              15                             4               0
                    // +---+--  --+---+-----+---+--  --+---+----+----+---+---+---+---+---+
                    // |   Reserved   | SGX |   Reserved   | SS | PK | I | R | U | W | P |
                    // +---+--  --+---+-----+---+--  --+---+----+----+---+---+---+---+---+
                    let mut map_flags = MappingFlags::USER; // TODO: add this flags through user tf.
                    if tf.error_code & (1 << 1) != 0 {
                        map_flags |= MappingFlags::WRITE;
                    }
                    if tf.error_code & (1 << 2) != 0 {
                        map_flags |= MappingFlags::USER;
                    }
                    if tf.error_code & (1 << 3) != 0 {
                        map_flags |= MappingFlags::READ;
                    }
                    if tf.error_code & (1 << 4) != 0 {
                        map_flags |= MappingFlags::EXECUTE;
                    }
                    axlog::debug!("error_code: {:?}", tf.error_code);
                    handle_page_fault(unsafe { cr2() }.into(), map_flags, tf);
                }
            } else {
                panic!(
                    "Kernel #PF @ {:#x}, fault_vaddr={:#x}, error_code={:#x}:\n{:#x?}",
                    tf.rip,
                    unsafe { cr2() },
                    tf.error_code,
                    tf,
                );
            }
        }
        BREAKPOINT_VECTOR => debug!("#BP @ {:#x} ", tf.rip),
        GENERAL_PROTECTION_FAULT_VECTOR => {
            panic!(
                "#GP @ {:#x}, error_code={:#x}:\n{:#x?}",
                tf.rip, tf.error_code, tf
            );
        }
        IRQ_VECTOR_START..=IRQ_VECTOR_END => crate::trap::handle_irq_extern(tf.vector as _),
        _ => {
            panic!(
                "Unhandled exception {} (error_code = {:#x}) @ {:#x}:\n{:#x?}",
                tf.vector, tf.error_code, tf.rip, tf
            );
        }
    }
    #[cfg(feature = "signal")]
    if tf.is_user() {
        crate::trap::handle_signal();
    }
}

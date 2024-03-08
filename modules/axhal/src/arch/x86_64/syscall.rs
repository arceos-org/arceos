use core::arch::global_asm;

use x86_64::{
    registers::{
        model_specific::{Efer, EferFlags, KernelGsBase, LStar, SFMask, Star},
        rflags::RFlags,
    },
    VirtAddr,
};

use crate::{arch::GdtStruct, trap::handle_syscall};

use super::TrapFrame;

#[cfg(feature = "monolithic")]
global_asm!(include_str!("syscall.S"));

#[no_mangle]
fn x86_syscall_handler(tf: &mut TrapFrame) {
    tf.rax = handle_syscall(tf.get_syscall_num(), tf.get_syscall_args()) as u64;
    #[cfg(feature = "signal")]
    crate::trap::handle_signal();
}

#[no_mangle]
#[percpu::def_percpu]
static USER_RSP_OFFSET: usize = 0;

#[no_mangle]
#[percpu::def_percpu]
static KERNEL_RSP_OFFSET: usize = 0;

pub fn init_syscall() {
    extern "C" {
        fn syscall_entry();
    }
    LStar::write(VirtAddr::new(syscall_entry as usize as _));
    Star::write(
        GdtStruct::UCODE64_SELECTOR,
        GdtStruct::UDATA_SELECTOR,
        GdtStruct::KCODE64_SELECTOR,
        GdtStruct::KDATA_SELECTOR,
    )
    .unwrap();
    SFMask::write(
        RFlags::TRAP_FLAG
            | RFlags::INTERRUPT_FLAG
            | RFlags::DIRECTION_FLAG
            | RFlags::IOPL_LOW
            | RFlags::IOPL_HIGH
            | RFlags::NESTED_TASK
            | RFlags::ALIGNMENT_CHECK,
    ); // TF | IF | DF | IOPL | AC | NT (0x47700)
    unsafe {
        Efer::update(|efer| *efer |= EferFlags::SYSTEM_CALL_EXTENSIONS);
    }
    KernelGsBase::write(VirtAddr::new(0));
}

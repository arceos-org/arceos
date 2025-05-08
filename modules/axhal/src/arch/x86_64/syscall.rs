use x86_64::addr::VirtAddr;
use x86_64::registers::model_specific::{Efer, EferFlags, KernelGsBase, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;
use x86_64::structures::tss::TaskStateSegment;

use super::{GdtStruct, TrapFrame};

#[unsafe(no_mangle)]
#[percpu::def_percpu]
static USER_RSP_OFFSET: usize = 0;

core::arch::global_asm!(
    include_str!("syscall.S"),
    tss_rsp0_offset = const core::mem::offset_of!(TaskStateSegment, privilege_stack_table),
    ucode64 = const GdtStruct::UCODE64_SELECTOR.0,
);

pub(super) fn handle_syscall(tf: &mut TrapFrame) {
    tf.rax = crate::trap::handle_syscall(tf, tf.rax as usize) as u64;
}

#[unsafe(no_mangle)]
fn x86_syscall_handler(tf: &mut TrapFrame) {
    super::tls::switch_to_kernel_fs_base(tf);
    handle_syscall(tf);
    crate::trap::post_trap_callback(tf, true);
    super::tls::switch_to_user_fs_base(tf);
}

/// Initializes syscall support and setups the syscall handler.
pub fn init_syscall() {
    unsafe extern "C" {
        fn syscall_entry();
    }
    unsafe {
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
        Efer::update(|efer| *efer |= EferFlags::SYSTEM_CALL_EXTENSIONS);
        KernelGsBase::write(VirtAddr::new(0));
    }
}

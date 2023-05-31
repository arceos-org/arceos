/// 将trap上下文从结构体写入到内核栈
use core::arch::global_asm;

use axhal::arch::TrapFrame;

global_asm!(include_str!("copy.S"));

extern "C" {
    pub fn __copy(frame_address: *mut TrapFrame, kernel_base: usize);
}

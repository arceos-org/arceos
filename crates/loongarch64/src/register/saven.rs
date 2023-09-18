// 数据保存控制状态寄存器用于给系统软件暂存数据。每个数据保存寄存器可以存放一个通用寄存器的数据。
// 数据保存寄存器最少实现 1 个，最多实现 16 个

use crate::register::csr::Register;
use core::arch::asm;

pub struct SaveReg0 {
    bits: usize,
}

impl Register for SaveReg0 {
    fn read() -> Self {
        unsafe {
            let mut bit: usize;
            asm!("csrrd {},0x30",out(reg) bit);
            Self { bits: bit }
        }
    }

    fn write(&mut self) {
        unsafe {
            asm!("csrwr {},0x30",in(reg) self.bits);
        }
    }
}

impl SaveReg0 {
    pub fn get_value(&self) -> usize {
        self.bits
    }
    pub fn set_value(&mut self, value: usize) -> &mut Self {
        self.bits = value;
        self
    }
}

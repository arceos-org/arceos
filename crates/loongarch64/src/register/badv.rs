use super::csr::Register;
use super::csr::CSR_BADV;
use core::arch::asm;
// 该寄存器用于触发地址错误相关例外时，记录出错的虚地址。此类例外包括：
#[derive(Debug)]
pub struct Badv {
    bits: usize,
}

impl Register for Badv {
    fn read() -> Self {
        let mut bits;
        // unsafe { asm!("csrrd {},{}", out(reg) bits,const CSR_BADV ) }
        unsafe { asm!("csrrd {},0x7", out(reg) bits ) }
        Self { bits }
    }
    fn write(&mut self) {
        // unsafe { asm!("csrwr {},{}",in(reg) self.bits, const CSR_BADV) }
        unsafe { asm!("csrwr {},0x7",in(reg) self.bits) }
    }
}

impl Badv {
    pub fn get_value(&self) -> usize {
        self.bits
    }
}

use super::csr::Register;
//use super::csr::CSR_BADI;
use core::arch::asm;
// 该寄存器用于记录触发同步类例外的指令的指令码。所谓同步类例外是指除了中断（INT）、客户机 CSR
// 硬件修改例外（GCHC）、机器错误例外（MERR）之外的所有例外。
#[derive(Debug)]
pub struct Badi {
    bits: usize,
}

impl Register for Badi {
    fn read() -> Self {
        let mut bits;
        //unsafe { asm!("csrrd {},{}", out(reg) bits, const CSR_BADI ) }
        unsafe { asm!("csrrd {},0x8", out(reg) bits ) }
        Self { bits }
    }

    fn write(&mut self) {}
}

impl Badi {
    pub fn get_val(&self) -> usize {
        self.bits
    }
}

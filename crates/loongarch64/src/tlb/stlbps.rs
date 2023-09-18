// 该寄存器用于配置 STLB 中页的大小

use super::super::register::csr::Register;
use super::super::register::csr::CSR_STLBPS;
use bit_field::BitField;
use core::arch::asm;
pub struct StlbPs {
    bits: u32,
}

impl Register for StlbPs {
    fn read() -> Self {
        let bits: u32;
        // unsafe { asm!("csrrd {},{}",out(reg)bits,const CSR_STLBPS) }
        unsafe { asm!("csrrd {},0x1e",out(reg)bits) }
        Self { bits }
    }
    fn write(&mut self) {
        // unsafe { asm!("csrwr {},{}",in(reg)self.bits,const CSR_STLBPS) }
        unsafe { asm!("csrwr {},0x1e",in(reg)self.bits) }
    }
}

impl StlbPs {
    pub fn get_page_size(&self) -> u32 {
        self.bits.get_bits(0..=5)
    }
    pub fn set_page_size(&mut self, page_size: u32) -> &mut Self {
        self.bits.set_bits(0..=5, page_size);
        self
    }
}

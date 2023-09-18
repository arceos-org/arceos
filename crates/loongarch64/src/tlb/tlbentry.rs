use super::super::register::csr::Register;
use super::super::register::csr::CSR_TLBRENTRY;
use core::arch::asm;

// TLB重填例外入口地址
pub struct TLBREntry {
    bits: usize,
}

impl Register for TLBREntry {
    fn read() -> Self {
        let bits: usize;
        unsafe {
            // asm!("csrrd {},{}", out(reg) bits,const CSR_TLBRENTRY);
            asm!("csrrd {},0x88", out(reg) bits);
        }
        TLBREntry { bits }
    }
    fn write(&mut self) {
        unsafe {
            // asm!("csrwr {},{}", in(reg) self.bits,const CSR_TLBRENTRY);
            asm!("csrwr {},0x88", in(reg) self.bits);
        }
    }
}

impl TLBREntry {
    pub fn get_val(&self) -> usize {
        self.bits
    }
    pub fn set_val(&mut self, val: usize) -> &mut Self {
        // 对齐到4kb
        assert!(val & 0xFFF == 0);
        self.bits = val;
        self
    }
}

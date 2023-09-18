use super::csr::{Register, CSR_TVAL};
use core::arch::asm;
pub struct Tval {
    bits: u32,
}

impl Register for Tval {
    fn read() -> Self {
        let mut tval;
        // unsafe { asm!("csrrd {},{}", out(reg)ticlr,const CSR_TICLR) }
        unsafe { asm!("csrrd {},0x42", out(reg)tval) }
        Tval { bits: tval }
    }
    fn write(&mut self) {}
}

impl Tval {
    pub fn get_val(&self) -> u32 {
        self.bits
    }
}

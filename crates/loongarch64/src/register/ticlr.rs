use super::csr::{Register, CSR_TICLR};
use bit_field::BitField;
use core::arch::asm;
pub struct Ticlr {
    bits: u32,
}

impl Register for Ticlr {
    fn read() -> Self {
        let mut ticlr;
        // unsafe { asm!("csrrd {},{}", out(reg)ticlr,const CSR_TICLR) }
        unsafe { asm!("csrrd {},0x44", out(reg)ticlr) }
        Ticlr { bits: ticlr }
    }
    fn write(&mut self) {
        // unsafe { asm!("csrwr {},{}", in(reg)self.bits,const CSR_TICLR) }
        unsafe { asm!("csrwr {},0x44", in(reg)self.bits) }
    }
}

impl Ticlr {
    pub fn set_val(&mut self, val: u32) -> &mut Self {
        self.bits = val;
        self
    }
    pub fn get_val(&self) -> u32 {
        self.bits
    }
    pub fn clear_timer(&mut self) -> &mut Self {
        self.bits.set_bit(0, true);
        self
    }
}

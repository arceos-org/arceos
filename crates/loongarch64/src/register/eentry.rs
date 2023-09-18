//例外入口地址

use super::csr::Register;
use core::arch::asm;
pub struct Eentry {
    bits: usize,
}
impl Register for Eentry {
    fn read() -> Self {
        let bits: usize;
        unsafe { asm!("csrrd {},0xc", out(reg) bits, ) }
        Eentry { bits }
    }
    fn write(&mut self) {
        unsafe { asm!("csrwr {},0xc", in(reg) self.bits, ) }
    }
}

impl Eentry {
    pub fn get_eentry(&self) -> usize {
        // 12位以后,以页对齐
        self.bits
    }
    pub fn set_eentry(&mut self, eentry: usize) -> &mut Self {
        assert!(eentry & 0xfff == 0);
        self.bits = eentry;
        self
    }
}

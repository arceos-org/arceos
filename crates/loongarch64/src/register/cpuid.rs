// 该寄存器中存有处理器核编号信息。

use super::csr::Register;
use super::csr::CSR_CPUID;
use core::arch::asm;
pub struct Cpuid {
    bits: u32,
}

impl Register for Cpuid {
    fn read() -> Self {
        let bits: u32;

        unsafe {
            //asm!("csrrd {},{}",out(reg) bits, const CSR_CPUID);
            asm!("csrrd {},0x20",out(reg) bits);
        }
        Self { bits }
    }
    fn write(&mut self) {}
}

impl Cpuid {
    pub fn get_val(&self) -> u32 {
        self.bits
    }
}

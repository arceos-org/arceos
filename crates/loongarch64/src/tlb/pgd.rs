// 该寄存器是一个只读寄存器，其内容是当前上下文中出错虚地址所对应的全局目录基址信息。该寄存
// 器的只读信息，不仅用于 CSR 类指令的读返回值，也用于 LDDIR 指令访问全局目录时所需的基址信息

use super::super::register::csr::Register;
use super::super::register::csr::CSR_PGD;
use core::arch::asm;

pub struct Pgd {
    pub pgd: usize,
}

impl Register for Pgd {
    fn read() -> Self {
        let bits: usize;
        unsafe {
            // asm!("csrrd {},{}",out(reg)bits,const CSR_PGD);
            asm!("csrrd {},0x1b",out(reg)bits);
        }
        Self { pgd: bits }
    }
    fn write(&mut self) {}
}

impl Pgd {
    pub fn get_val(&self) -> usize {
        self.pgd
    }
}

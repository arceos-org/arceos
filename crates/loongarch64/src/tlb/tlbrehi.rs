// 无论 CSR.TLBRERA.IsTLBR 等于何值，执行 TLBRD 指令都只更新 TLBEHI 寄存器

use super::super::register::csr::Register;
use super::super::register::csr::CSR_TLBREHI;
use bit_field::BitField;
use core::arch::asm;

pub struct TlbREhi {
    bits: u64,
}

impl Register for TlbREhi {
    fn read() -> Self {
        let bits: u64;
        unsafe {
            // asm!("csrrd {},{}",out(reg)bits,const CSR_TLBREHI);
            asm!("csrrd {},0x8E",out(reg)bits);
        }
        Self { bits }
    }
    fn write(&mut self) {
        unsafe {
            // asm!("csrwr {},{}",in(reg)self.bits,const CSR_TLBREHI);
            asm!("csrwr {},0x8E",in(reg)self.bits);
        }
    }
}

impl TlbREhi {
    // TLB 重填例外专用的页大小值。即在 CSR.TLBRERA.IsTLBR=1 时，执行 TLBWR 和 TLBFILL
    // 指令，写入的 TLB 表项的 PS 域的值来自于此。
    pub fn get_page_size(&self) -> u64 {
        self.bits.get_bits(0..=5)
    }
    pub fn set_page_size(&mut self, page_size: u64) -> &mut Self {
        self.bits.set_bits(0..=5, page_size);
        self
    }
    pub fn get_vppn(&self, valen: usize) -> u64 {
        self.bits.get_bits(13..valen)
    }
    pub fn set_vppn(&mut self, valen: usize, vppn: u64) -> &mut Self {
        self.bits.set_bits(13..valen, vppn);
        self
    }
}

use super::super::register::csr::Register;
use super::super::register::csr::CSR_PWCH;
use bit_field::BitField;
use core::arch::asm;
pub struct Pwch {
    bits: u32,
}
impl Register for Pwch {
    fn read() -> Self {
        let bits: u32;
        // unsafe { asm!("csrrd {},{}",out(reg)bits,const CSR_PWCH) }
        unsafe { asm!("csrrd {},0x1d",out(reg)bits) }
        Self { bits }
    }
    fn write(&mut self) {
        // unsafe { asm!("csrwr {},{}",in(reg)self.bits,const CSR_PWCH) }
        unsafe { asm!("csrwr {},0x1d",in(reg)self.bits) }
    }
}
impl Pwch {
    //次高一级目录的起始地址
    pub fn get_dir3_base(&self) -> u32 {
        self.bits.get_bits(0..=5)
    }
    pub fn set_dir3_base(&mut self, dir2_base: u32) -> &mut Self {
        self.bits.set_bits(0..=5, dir2_base);
        self
    }
    // 次高一级目录的索引位数。0 表示没有这一级。
    pub fn get_dir3_width(&self) -> u32 {
        self.bits.get_bits(6..=11)
    }
    pub fn set_dir3_width(&mut self, dir2_width: u32) -> &mut Self {
        self.bits.set_bits(6..=11, dir2_width);
        self
    }
    pub fn get_dir4_base(&self) -> u32 {
        self.bits.get_bits(12..=17)
    }
    pub fn set_dir4_base(&mut self, dir3_base: u32) -> &mut Self {
        self.bits.set_bits(12..=17, dir3_base);
        self
    }
    pub fn get_dir4_width(&self) -> u32 {
        self.bits.get_bits(18..=23)
    }
    pub fn set_dir4_width(&mut self, dir3_width: u32) -> &mut Self {
        self.bits.set_bits(18..=23, dir3_width);
        self
    }
}

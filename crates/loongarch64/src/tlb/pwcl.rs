// 该寄存器和 CSR.PWCH 寄存器中的信息在一起定义了操作系统中所采用的页表结构。这些信息将用于指
// 示软件或硬件进行页表遍历
// 在 LA32 架构下仅实现 CSR.PWCL。为此 PWCL 寄存器需包含刻画页表结构的所有信息，由此导致末级页
// 表和最低两级目录的起始地址位置均不超过 32 位，该限制在 LA64 架构下依然存在

use super::super::register::csr::Register;
use super::super::register::csr::CSR_PWCL;
use bit_field::BitField;
use core::arch::asm;
pub struct Pwcl {
    bits: u32,
}

impl Register for Pwcl {
    fn read() -> Self {
        let bits: u32;
        // unsafe { asm!("csrrd {},{}",out(reg)bits,const CSR_PWCL) }
        unsafe { asm!("csrrd {},0x1c",out(reg)bits) }
        Self { bits }
    }
    fn write(&mut self) {
        // unsafe { asm!("csrwr {},{}",in(reg)self.bits,const CSR_PWCL) }
        unsafe { asm!("csrwr {},0x1c",in(reg)self.bits) }
    }
}
impl Pwcl {
    // 末级页表的起始地址。
    pub fn get_ptbase(&self) -> u32 {
        self.bits.get_bits(0..=4)
    }
    pub fn set_ptbase(&mut self, ptbase: u32) -> &mut Self {
        self.bits.set_bits(0..=4, ptbase);
        self
    }
    // 末级页表的索引位数
    pub fn get_ptwidth(&self) -> u32 {
        self.bits.get_bits(5..=9)
    }
    pub fn set_ptwidth(&mut self, ptwidth: u32) -> &mut Self {
        self.bits.set_bits(5..=9, ptwidth);
        self
    }
    pub fn get_dir1_base(&self) -> u32 {
        self.bits.get_bits(10..=14)
    }
    pub fn set_dir1_base(&mut self, dir1_base: u32) -> &mut Self {
        self.bits.set_bits(10..=14, dir1_base);
        self
    }
    // 最低一级目录的索引位数。0 表示没有这一级
    pub fn get_dir1_width(&self) -> u32 {
        self.bits.get_bits(15..=19)
    }
    pub fn set_dir1_width(&mut self, dir1_width: u32) -> &mut Self {
        self.bits.set_bits(15..=19, dir1_width);
        self
    }
    pub fn get_dir2_base(&self) -> u32 {
        self.bits.get_bits(20..=24)
    }
    pub fn set_dir2_base(&mut self, dir2_base: u32) -> &mut Self {
        self.bits.set_bits(20..=24, dir2_base);
        self
    }
    // 最低两级目录的索引位数。0 表示没有这一级
    pub fn get_dir2_width(&self) -> u32 {
        self.bits.get_bits(25..=29)
    }
    pub fn set_dir2_width(&mut self, dir2_width: u32) -> &mut Self {
        self.bits.set_bits(25..=29, dir2_width);
        self
    }
    // 0 表示 64 比特，1 表示 128 比特，2 表示 192 比特，3 表示 256 比特。
    pub fn get_pte_width(&self) -> u32 {
        let val = self.bits.get_bits(30..=31);
        match val {
            0 => 64,
            1 => 128,
            2 => 192,
            3 => 256,
            _ => panic!("invalid pte_width"),
        }
    }
    pub fn set_pte_width(&mut self, pte_width: u32) -> &mut Self {
        let val = match pte_width {
            64 => 0,
            128 => 1,
            192 => 2,
            256 => 3,
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 3,
            _ => panic!("invalid pte_width"),
        };
        self.bits.set_bits(30..=31, val);
        self
    }
}

use super::csr::{Register, CSR_PRCFG1};
use bit_field::BitField;
use core::arch::asm;
#[derive(Debug)]
pub struct Prcfg1 {
    // [0-3] Savenum 保存数据寄存器的数量
    // [4-11] TimerBits 定时器的位数-1
    // [12-14] 例外入口地址间距的 vs可以设置的最大值
    // [15-31] 0
    bits: usize,
}

impl Register for Prcfg1 {
    fn read() -> Self {
        let mut bits;
        // unsafe { asm!("csrrd {},{}", out(reg) bits,const CSR_PRCFG1 ) }
        unsafe { asm!("csrrd {},0x21", out(reg) bits ) }
        Self { bits }
    }
    fn write(&mut self) {}
}

impl Prcfg1 {
    pub fn get_save_num(&self) -> usize {
        self.bits.get_bits(0..4)
    }
    pub fn get_timer_bits(&self) -> usize {
        // 返回定时器的位数
        self.bits.get_bits(4..12) + 1
    }
    pub fn get_vs_max(&self) -> usize {
        self.bits.get_bits(12..15)
    }
}

// 当触发 TLB 重填例外时，硬件会将此时处理器核的特权等级、客户机模式、全局中断使能和监视点使
// 能位保存至该寄存器中，用于例外返回时恢复处理器核的现场

use super::super::cpu::CpuMode;
use super::super::register::csr::Register;
use super::super::register::csr::CSR_TLBRPRMD;
use bit_field::BitField;
use core::arch::asm;
pub struct TlbRPrmd {
    bits: u32,
}

impl Register for TlbRPrmd {
    fn read() -> Self {
        let bits: u32;
        unsafe {
            // asm!("csrrd {},{}", out(reg) bits,const CSR_TLBRPRMD);
            asm!("csrrd {},0x8f", out(reg) bits);
        }
        Self { bits }
    }
    fn write(&mut self) {
        //写入era的内容
        unsafe {
            // asm!("csrwr {},{}", in(reg) self.bits,const CSR_TLBRPRMD);
            asm!("csrwr {},0x8f", in(reg) self.bits);
        }
    }
}

impl TlbRPrmd {
    pub fn get_pplv(&self) -> u32 {
        self.bits.get_bits(0..2)
    }
    pub fn set_pplv(&mut self, pplv: CpuMode) -> &mut Self {
        //设置特权级
        // 用于在进入用户程序时设置特权级
        self.bits.set_bits(0..2, pplv as u32);
        self
    }
    // 记录例外发生前的crmd.ie
    pub fn get_pie(&self) -> bool {
        self.bits.get_bit(2)
    }
    // 设置中断使能
    // 用于在进入用户程序时设置中断使能
    pub fn set_pie(&mut self, pie: bool) -> &mut Self {
        self.bits.set_bit(2, pie);
        self
    }

    pub fn get_pwe(&self) -> bool {
        self.bits.get_bit(4)
    }
    pub fn set_pwe(&mut self, pwe: bool) -> &mut Self {
        self.bits.set_bit(4, pwe);
        self
    }
}

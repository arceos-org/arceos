use super::super::cpu::CpuMode;
use super::csr::{Register, CSR_PRMD};
use bit_field::BitField;
use core::arch::asm;
// 当触发例外时，如果例外类型不是 TLB 重填例外和机器错误例外，硬件会将此时处理器核的特权等级、
// 全局中断使能和监视点使能位保存至例外前模式信息寄存器中，用于例外返回时恢复处理器核的现场
pub struct Prmd {
    bits: usize,
}

impl Register for Prmd {
    fn read() -> Self {
        let mut bits;
        // unsafe { asm!("csrrd {},{}", out(reg) bits ,const CSR_PRMD) }
        unsafe { asm!("csrrd {},0x1", out(reg) bits) }
        Self { bits }
    }
    fn write(&mut self) {
        // unsafe { asm!("csrwr {},{}", in(reg) self.bits ,const CSR_PRMD) }
        unsafe { asm!("csrwr {},0x1", in(reg) self.bits) }
    }
}

impl Prmd {
    pub fn get_val(&self) -> usize {
        self.bits
    }
    pub fn set_val(&mut self, val: usize) -> &mut Self {
        self.bits = val;
        self
    }

    // 当触发例外时，如果例外类型不是 TLB 重填例外和机器错误例外，硬件会将 CSR.CRMD
    // 中 PLV 域的旧值记录在这个域。
    // 当所处理的例外既不是 TLB 重填例外（CSR.TLBRERA.IsTLBR=0）也不是机器错误例外
    // （CSR.ERRCTL.IsMERR=0）时，执行 ERTN 指令从例外处理程序返回时，硬件会将这个
    // 域的值恢复到 CSR.CRMD 的 PLV 域
    pub fn get_pplv(&self) -> usize {
        self.bits.get_bits(0..2)
    }
    pub fn set_pplv(&mut self, pplv: CpuMode) -> &mut Self {
        //设置特权级
        // 用于在进入用户程序时设置特权级
        self.bits.set_bits(0..2, pplv as usize);
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
        self.bits.get_bit(3)
    }
    pub fn set_pwe(&mut self, pwe: bool) -> &mut Self {
        self.bits.set_bit(3, pwe);
        self
    }
}

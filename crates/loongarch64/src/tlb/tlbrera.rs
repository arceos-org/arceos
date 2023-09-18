// 该寄存器保存 TLB 重填例外处理完毕之后的返回地址。除此之外，该寄存器还包含用于标识当前例外
// 是 TLB 重填例外的标志位

use super::super::register::csr::Register;
use super::super::register::csr::CSR_TLBRERA;
use bit_field::BitField;
use core::arch::asm;
pub struct TLBREra {
    bits: usize,
}

impl Register for TLBREra {
    fn read() -> Self {
        let mut bits;
        unsafe {
            // asm!("csrrd {},{}", out(reg) bits,const CSR_TLBRERA);
            asm!("csrrd {},0x8A", out(reg) bits);
        }
        TLBREra { bits }
    }
    fn write(&mut self) {
        //写入era的内容
        unsafe {
            // asm!("csrwr {},{}", in(reg) self.bits,const CSR_TLBRERA);
            asm!("csrwr {},0x8A", in(reg) self.bits);
        }
    }
}

impl TLBREra {
    // 记录触发 TLB 重填例外的指令的 PC 的[GRLEN-1:2]位。当执行 ERTN 指令从 TLB 重填
    // 例外处理程序返回时（此时本寄存器 IsTLBR=1 且 CSR.ERRCTL.IsMERR=0），硬件自动
    // 将存放在此处的值最低位补上两比特 0 后作为最终的返回地址
    pub fn get_pc(&self) -> usize {
        // 返回pc
        self.bits.get_bits(2..)
    }
    pub fn get_is_tlbr(&self) -> bool {
        // 返回是否是 TLB 重填例外
        self.bits.get_bit(0)
    }
    pub fn set_is_tlbr(&mut self, is_tlbr: bool) -> &mut Self {
        self.bits.set_bit(0, is_tlbr);
        self
    }
}

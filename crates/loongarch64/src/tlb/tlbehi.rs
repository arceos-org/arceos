// 该寄存器包含 TLB 指令操作时与 TLB 表项高位部分虚页号相关的信息。因 TLB 表项高位所含的 VPPN 域
// 的位宽与实现所支持的有效虚地址范围相关，故有关寄存器域的定义分开表述

use super::super::register::csr::Register;
use super::super::register::csr::CSR_TLBEHI;
use bit_field::BitField;
use core::arch::asm;

pub struct TlbEHi {
    bits: usize,
}
impl Register for TlbEHi {
    fn read() -> Self {
        let bits: usize;
        // unsafe { asm!("csrrd {},{}",out(reg)bits,const CSR_TLBEHI) }
        unsafe { asm!("csrrd {},0x11",out(reg)bits) }
        TlbEHi { bits }
    }
    fn write(&mut self) {
        // unsafe { asm!("csrwr {},{}",in(reg)self.bits,const CSR_TLBEHI) }
        unsafe { asm!("csrwr {},0x11",in(reg)self.bits) }
    }
}
impl TlbEHi {
    // 执行 TLBRD 指令时，所读取 TLB 表项的 VPPN 域的值记录到这里。
    // 在 CSR.TLBRERA.IsTLBR=0 时，执行 TLBSRCH 指令时查询 TLB 所用 VPPN 值，以及执行
    // TLBWR 和 TLBFILL 指令时写入 TLB 表项的 VPPN 域的值来自于此。
    // 当触发 load 操作页无效例外、store 操作页无效例外、取指操作页无效例外、页修
    // 改例外、页不可读例外、页不可执行例外和页特权等级不合规例外时，触发例外的地址的[VALEN-1:13]位被记录到这里。
    pub fn get_vppn(&self, valen: usize) -> usize {
        self.bits.get_bits(13..valen)
    }
    pub fn set_vppn(&mut self, valen: usize, vppn: usize) -> &mut Self {
        self.bits.set_bits(13..valen, vppn);
        self
    }
}

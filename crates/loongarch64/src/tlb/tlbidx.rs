// 该寄存器包含 TLB 指令操作 TLB 时相关的索引值等信息。Index 域的位宽与实现相关，不
// 过本架构所允许的 Index 位宽不超过 16 比特。
// 该寄存器还包含 TLB 指令操作时与 TLB 表项中 PS、E 域相关的信息

use super::super::register::csr::Register;
use super::super::register::csr::CSR_TLBIDX;
use bit_field::BitField;
use core::arch::asm;
pub struct TlbIdx {
    bits: u32,
}
impl Register for TlbIdx {
    fn read() -> Self {
        let bits: u32;
        // unsafe { asm!("csrrd {},{}",out(reg)bits,const CSR_TLBIDX) }
        unsafe { asm!("csrrd {},0x10",out(reg)bits) }
        Self { bits }
    }
    fn write(&mut self) {
        // unsafe { asm!("csrwr {},{}",in(reg)self.bits,const CSR_TLBIDX) }
        unsafe { asm!("csrwr {},0x10",in(reg)self.bits) }
    }
}
impl TlbIdx {
    // 执行 TLBRD 和 TLBWR 指令时，访问 TLB 表项的索引值来自于此。
    // 执行 TLBSRCH 指令时，如果命中，则命中项的索引值记录到这里
    pub fn get_index(&self) -> u32 {
        self.bits.get_bits(0..16)
    }
    pub fn set_index(&mut self, index: u32) -> &mut Self {
        self.bits.set_bits(0..16, index);
        self
    }
    // 执行 TLBRD 指令时，所读取 TLB 表项的 PS 域的值记录到这里。
    // 在 CSR.TLBRERA.IsTLBR=0 时，执行 TLBWR 和 TLBFILL 指令，写入的 TLB 表项的 PS
    // 域的值来自于此。
    pub fn get_ps(&self) -> u32 {
        self.bits.get_bits(24..=29)
    }
    pub fn set_ps(&mut self, ps: u32) -> &mut Self {
        self.bits.set_bits(24..=29, ps);
        self
    }
    // 该位为 1 表示该 TLB 表项为空（无效 TLB 表项），为 0 表示该 TLB 表项非空（有效 TLB
    // 表项）。
    // 执行 TLBSRCH 时，如果有命中项该位记为 0，否则该位记为 1。
    // 执行 TLBRD 时，所读取 TLB 表项的 E 位信息取反后记录到这里。
    // 执行 TLBWR 或 TLBFILL 指令时，若 CSR.TLBRERA.IsTLBR=0，将该位的值取反后写入
    // 到被写 TLB 项的 E 位；若此时 CSR.TLBRERA.IsTLBR=1，那么被写入的 TLB 项的 E 位
    // 总是置为 1，与该位的值无关
    pub fn get_ne(&self) -> bool {
        self.bits.get_bit(31)
    }
    pub fn set_ne(&mut self, ne: bool) -> &mut Self {
        self.bits.set_bit(31, ne);
        self
    }
}

use super::csr::Register;
use super::csr::CSR_PRCFG3;
use bit_field::BitField;
use core::arch::asm;
// 该寄存器包含一些特权资源的配置信息。
pub struct Prcfg3 {
    bits: u32,
}
impl Register for Prcfg3 {
    fn read() -> Self {
        let bits: u32;
        unsafe {
            // asm!("csrrd {},{}",out(reg) bits,const CSR_PRCFG3);
            asm!("csrrd {},0x23",out(reg) bits);
        }
        Self { bits }
    }
    fn write(&mut self) {}
}

impl Prcfg3 {
    // 指示 TLB 组织方式：
    //     0：没有 TLB；
    //     1：一个全相联的多重页大小 TLB（MTLB）
    //     2：一个全相联的多重页大小 TLB（MTLB）+一个组相联的单个页大小 TLB（STLB）；
    //     其它值：保留。

    pub fn get_tlb_type(&self) -> u32 {
        self.bits.get_bits(0..=3)
    }
    // 当 TLBType=1 或 2 时，该域的值是全相联多重页大小 TLB 的项数减 1
    pub fn get_mtlb_entries(&self) -> u32 {
        self.bits.get_bits(4..=11)
    }

    // STLBWays
    pub fn get_stlb_ways(&self) -> u32 {
        self.bits.get_bits(12..=19) + 1
    }

    // 当 TLBType=2 时，该域的值是组相联单个页大小 TLB 的每一路项数的幂指数，即每一
    // 路有 2 ^ STLBSets项。
    pub fn get_sltb_sets(&self) -> u32 {
        self.bits.get_bits(20..=25)
    }
}

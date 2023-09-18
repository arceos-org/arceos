use super::super::register::csr::CSR_TLBELO;
use super::super::PALEN;
use bit_field::BitField;
use core::arch::asm;
use core::fmt;
// TLBELO0 和 TLBELO1 两个寄存器包含了 TLB 指令操作时 TLB 表项低位部分物理页号等相关的信息。因龙
// 芯架构下 TLB 采用双页结构，所以 TLB 表项的低位信息对应奇偶两个物理页表项，其中偶数页信息在 TLBELO0
// 中，奇数页信息在 TLBELO1 中。
pub trait TLBEL {
    fn get_valid(&self) -> bool;
    fn set_valid(&mut self, valid: bool) -> &mut Self;
    fn get_dirty(&self) -> bool;
    fn set_dirty(&mut self, dirty: bool) -> &mut Self;
    fn get_plv(&self) -> usize;
    fn set_plv(&mut self, plv: usize) -> &mut Self;
    fn get_mem_access_type(&self) -> usize;
    fn set_mem_access_type(&mut self, mem_access_type: usize) -> &mut Self;
    fn get_global_flag(&self) -> bool;
    fn set_global_flag(&mut self, global_flag: bool) -> &mut Self;
    fn get_ppn(&self, paleln: usize) -> usize;
    fn set_ppn(&mut self, palen: usize, ppn: usize) -> &mut Self;
    fn get_not_readable(&self) -> bool;
    fn set_not_readable(&mut self, not_readable: bool) -> &mut Self;
    fn get_not_executable(&self) -> bool;
    fn set_not_executable(&mut self, not_executable: bool) -> &mut Self;
    fn get_rplv(&self) -> bool;
    fn set_rplv(&mut self, rplv: bool) -> &mut Self;
}
pub struct TLBELO {
    bits: usize,
    index: usize,
}

impl fmt::Debug for TLBELO {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TlbElo{}: RPLV:{},NX:{},NR:{},PPN:{:#x},G:{},MAT:{},PLV:{},D:{},V:{}",
            self.index,
            self.get_rplv(),
            self.get_not_executable(),
            self.get_not_readable(),
            self.get_ppn(PALEN),
            self.get_global_flag(),
            self.get_mem_access_type(),
            self.get_plv(),
            self.get_dirty(),
            self.get_valid()
        )
    }
}

impl TLBELO {
    pub fn read(index: usize) -> Self {
        let bits: usize;
        unsafe {
            match index {
                // 0 => asm!("csrrd {},{}",out(reg)bits,const CSR_TLBELO),
                // 1 => asm!("csrrd {},{}",out(reg)bits,const CSR_TLBELO+1),
                0 => asm!("csrrd {},0x12",out(reg)bits),
                1 => asm!("csrrd {},0x13",out(reg)bits),
                _ => panic!("TLBELO index out of range"),
            }
        }
        Self { bits, index }
    }
    pub fn write(&mut self) {
        unsafe {
            match self.index {
                // 0 => asm!("csrwr {},{}",in(reg)self.bits,const CSR_TLBELO),
                // 1 => asm!("csrwr {},{}",in(reg)self.bits,const CSR_TLBELO+1),
                0 => asm!("csrwr {},0x12",in(reg)self.bits),
                1 => asm!("csrwr {},0x13",in(reg)self.bits),
                _ => panic!("TLBELO index out of range"),
            }
        }
    }
}

impl TLBELO {
    pub fn set_val(&mut self, val: usize) -> &mut Self {
        self.bits = val;
        self
    }
}
impl TLBEL for TLBELO {
    // 页表项的有效位（V）
    fn get_valid(&self) -> bool {
        self.bits.get_bit(0)
    }

    fn set_valid(&mut self, valid: bool) -> &mut Self {
        self.bits.set_bit(0, valid);
        self
    }

    fn get_dirty(&self) -> bool {
        self.bits.get_bit(1)
    }

    fn set_dirty(&mut self, dirty: bool) -> &mut Self {
        self.bits.set_bit(1, dirty);
        self
    }

    fn get_plv(&self) -> usize {
        self.bits.get_bits(2..=3)
    }

    fn set_plv(&mut self, plv: usize) -> &mut Self {
        self.bits.set_bits(2..=3, plv);
        self
    }

    fn get_mem_access_type(&self) -> usize {
        self.bits.get_bits(4..=5)
    }

    fn set_mem_access_type(&mut self, mem_access_type: usize) -> &mut Self {
        self.bits.set_bits(4..=5, mem_access_type);
        self
    }

    fn get_global_flag(&self) -> bool {
        self.bits.get_bit(6)
    }

    fn set_global_flag(&mut self, global_flag: bool) -> &mut Self {
        self.bits.set_bit(6, global_flag);
        self
    }

    fn get_ppn(&self, palen: usize) -> usize {
        self.bits.get_bits(12..palen)
    }

    fn set_ppn(&mut self, palen: usize, ppn: usize) -> &mut Self {
        self.bits.set_bits(12..palen, ppn);
        self
    }

    fn get_not_readable(&self) -> bool {
        self.bits.get_bit(61)
    }

    fn set_not_readable(&mut self, not_readable: bool) -> &mut Self {
        self.bits.set_bit(61, not_readable);
        self
    }

    fn get_not_executable(&self) -> bool {
        self.bits.get_bit(62)
    }

    fn set_not_executable(&mut self, not_executable: bool) -> &mut Self {
        self.bits.set_bit(62, not_executable);
        self
    }

    fn get_rplv(&self) -> bool {
        self.bits.get_bit(63)
    }

    fn set_rplv(&mut self, rplv: bool) -> &mut Self {
        self.bits.set_bit(63, rplv);
        self
    }
}

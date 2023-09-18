use super::super::register::csr::Register;
use super::super::register::csr::CSR_ASID;
use bit_field::BitField;
use core::arch::asm;

// 该寄存器中包含了用于访存操作和 TLB 指令的地址空间标识符（ASID）信息。ASID 的位宽随着架构规
// 范的演进可能进一步增加，为方便软件明确 ASID 的位宽，将直接给出这一信息
pub struct Asid {
    bits: u32,
}

impl Register for Asid {
    fn read() -> Self {
        let bits: u32;
        // unsafe { asm!("csrrd {},{}",out(reg)bits,const CSR_ASID) }
        unsafe { asm!("csrrd {},0x18",out(reg)bits) }
        Self { bits }
    }
    fn write(&mut self) {
        //unsafe { asm!("csrwr {},{}", in(reg)self.bits,const CSR_ASID) }
        unsafe { asm!("csrwr {},0x18", in(reg)self.bits) }
    }
}
impl Asid {
    pub fn get_asid(&self) -> u32 {
        self.bits.get_bits(0..10)
    }
    pub fn set_asid(&mut self, asid: u32) -> &mut Self {
        self.bits.set_bits(0..10, asid);
        self
    }

    pub fn get_asid_width(&self) -> u32 {
        self.bits.get_bits(16..=23)
    }
    pub fn set_asid_width(&mut self, asid_width: u32) -> &mut Self {
        self.bits.set_bits(16..=23, asid_width);
        self
    }
}

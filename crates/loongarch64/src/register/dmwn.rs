use super::csr::Register;
use bit_field::BitField;
use core::arch::asm;
// 直接地址映射配置窗口
pub struct Dmw0 {
    bits: usize,
}
pub struct Dmw1 {
    bits: usize,
}
pub struct Dmw2 {
    bits: usize,
}
pub struct Dmw3 {
    bits: usize,
}

impl Register for Dmw0 {
    fn read() -> Self {
        let mut bits: usize;
        unsafe { asm!("csrrd {},0x180", out(reg) bits ) }
        Dmw0 { bits }
    }
    fn write(&mut self) {
        unsafe { asm!("csrwr {},0x180", in(reg) self.bits ) }
    }
}
impl Dmw0 {
    pub fn get_value(&self) -> usize {
        self.bits
    }
    pub fn set_value(&mut self, value: usize) -> &mut Self {
        self.bits = value;
        self
    }
    pub fn set_plv_with_level(&mut self, level: usize, enable: bool) -> &mut Self {
        assert!(level <= 3);
        self.bits.set_bit(level, enable);
        self
    }
    pub fn set_mat(&mut self, enable: bool) -> &mut Self {
        self.bits.set_bit(4, enable);
        self
    }
    pub fn set_vesg(&mut self, value: usize) -> &mut Self {
        self.bits.set_bits(60..64, value);
        self
    }
}

impl Register for Dmw1 {
    fn read() -> Self {
        let mut bits: usize;
        unsafe { asm!("csrrd {},0x181", out(reg) bits ) }
        Dmw1 { bits }
    }
    fn write(&mut self) {
        unsafe { asm!("csrwr {},0x181", in(reg) self.bits ) }
    }
}
impl Dmw1 {
    pub fn get_value(&self) -> usize {
        self.bits
    }
    pub fn set_value(&mut self, value: usize) -> &mut Self {
        self.bits = value;
        self
    }
}

impl Register for Dmw2 {
    fn read() -> Self {
        let mut bits: usize;
        unsafe { asm!("csrrd {},0x182", out(reg) bits ) }
        Dmw2 { bits }
    }
    fn write(&mut self) {
        unsafe { asm!("csrwr {},0x182", in(reg) self.bits ) }
    }
}
impl Dmw2 {
    pub fn get_value(&self) -> usize {
        self.bits
    }
    pub fn set_value(&mut self, value: usize) -> &mut Self {
        self.bits = value;
        self
    }
}

impl Register for Dmw3 {
    fn read() -> Self {
        let mut bits: usize;
        unsafe { asm!("csrrd {},0x183", out(reg) bits ) }
        Dmw3 { bits }
    }
    fn write(&mut self) {
        unsafe { asm!("csrwr {},0x183", in(reg) self.bits ) }
    }
}
impl Dmw3 {
    pub fn get_value(&self) -> usize {
        self.bits
    }
    pub fn set_value(&mut self, value: usize) -> &mut Self {
        self.bits = value;
        self
    }
}

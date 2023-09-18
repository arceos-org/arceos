use super::super::cpu::CpuMode;
use super::csr::Register;
use super::csr::CSR_CRMD;
use bit_field::BitField;
use core::arch::asm;
// 当前模式信息
#[repr(C)]
pub struct Crmd {
    bits: usize,
}

impl Register for Crmd {
    fn read() -> Self {
        //读取crmd的内容
        let mut crmd;
        unsafe {
            // asm!("csrrd {},{}", out(reg) crmd,const CSR_CRMD);
            asm!("csrrd {},0x0", out(reg) crmd);
        }
        Crmd { bits: crmd }
    }
    fn write(&mut self) {
        //写入crmd
        unsafe {
            // asm!("csrwr {},{}", in(reg) self.bits,const CSR_CRMD);
            asm!("csrwr {},0x0", in(reg) self.bits);
        }
    }
}

impl Crmd {
    // 返回整个寄存器的内容
    pub fn get_val(&self) -> usize {
        self.bits
    }
    pub fn set_val(&mut self, val: usize) -> &mut Self {
        self.bits = val;
        self
    }
    // 返回当前特权级模式
    // 0-1位
    pub fn get_plv(&self) -> usize {
        self.bits.get_bits(0..2)
    }
    // 设置特权级模式
    pub fn set_plv(&mut self, mode: CpuMode) -> &mut Self {
        self.bits.set_bits(0..2, mode as usize);
        self
    }
    // 设置全局中断使能
    // 第2位
    pub fn set_ie(&mut self, enable: bool) -> &mut Self {
        self.bits.set_bit(2, enable);
        self
    }
    // 获取全局中断使能
    pub fn get_ie(&self) -> bool {
        self.bits.get_bit(2)
    }
    // 获取DA
    pub fn get_da(&self) -> bool {
        // 第3位
        self.bits.get_bit(3)
    }
    // 设置DA,直接地址翻译使能
    pub fn set_da(&mut self, da: bool) -> &mut Self {
        self.bits.set_bit(3, da);
        self
    }
    // 获取PG
    // 第4位
    pub fn get_pg(&self) -> bool {
        self.bits.get_bit(4)
    }
    // 设置PG,页翻译使能
    pub fn set_pg(&mut self, pg: bool) -> &mut Self {
        self.bits.set_bit(4, pg);
        self
    }
    // 获取直接地址翻译模式时，取指操作的存储访问类型
    // 在采用软件处理 TLB 重填的情况下，当软件将 PG 置为 1 时，需同时将 DATF 域置为
    // 0b01，即一致可缓存类型
    pub fn get_datf(&self) -> usize {
        self.bits.get_bits(5..=6)
    }
    pub fn set_datf(&mut self, datf: usize) -> &mut Self {
        self.bits.set_bits(5..=6, datf);
        self
    }
    // 直接地址翻译模式时，load 和 store 操作的存储访问类型
    pub fn get_datm(&self) -> usize {
        self.bits.get_bits(7..=8)
    }
    pub fn set_datm(&mut self, datm: usize) -> &mut Self {
        self.bits.set_bits(7..=8, datm);
        self
    }
}

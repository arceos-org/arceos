use super::csr::Register;
use bit_field::BitField;
use core::arch::asm;

// 定时器寄存器
// 该寄存器是软件配置定时器的接口。定时器的有效位数由实现决定，因此该寄存器中 TimeVal 域的位
// 宽也将随之变化。
#[derive(Debug)]
pub struct Tcfg {
    // [0] 使能位
    // [1] 循环控制位
    // [2-]计时器数值， 为4的整数倍
    bits: usize,
}

impl Register for Tcfg {
    fn read() -> Self {
        let mut tcfg;
        unsafe { asm!("csrrd {} , 0x41", out(reg) tcfg, ) }
        Self { bits: tcfg }
    }
    fn write(&mut self) {
        unsafe { asm!("csrwr {}, 0x41", in(reg) self.bits, ) }
    }
}

impl Tcfg {
    pub fn get_enable(&self) -> bool {
        //第0位
        !self.bits.get_bit(0)
    }
    pub fn set_enable(&mut self, enable: bool) -> &mut Self {
        self.bits.set_bit(0, enable);
        self
    }
    pub fn get_loop(&self) -> bool {
        //第1位
        self.bits.get_bit(1)
    }
    pub fn set_loop(&mut self, loop_: bool) -> &mut Self {
        self.bits.set_bit(1, loop_);
        self
    }
    pub fn get_initval(&self) -> usize {
        //第2位开始
        (self.bits >> 2) << 2
    }
    pub fn set_initval(&mut self, val: usize) -> &mut Self {
        // 设置计数值, 只能是4的整数倍
        // 在数值末尾会补上2bit0
        self.bits.set_bits(2.., val >> 2);
        self
    }
    pub fn get_val(&self) -> usize {
        self.bits
    }
    pub fn set_val(&mut self, val: usize) -> &mut Self {
        self.bits = val;
        self
    }
}

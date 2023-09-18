use super::csr::Register;
use super::csr::CSR_RVACFG;
use core::arch::asm;
// 该寄存器用于控制虚地址缩减模式下被缩减的地址位宽。
pub struct RvaCfg {
    bits: u32,
}

impl Register for RvaCfg {
    fn read() -> Self {
        let bits: u32;
        unsafe {
            // asm!("csrrd {},{}",out(reg) bits, const CSR_RVACFG);
            asm!("csrrd {},0x1F",out(reg) bits);
        }
        Self { bits }
    }
    fn write(&mut self) {
        unsafe {
            // asm!("csrwr {},{}",in(reg) self.bits, const CSR_RVACFG);
            asm!("csrwr {},0x1F",in(reg) self.bits);
        }
    }
}

impl RvaCfg {
    fn get_val(&self) -> u32 {
        self.bits
    }
    // 虚地址缩减模式下，被缩减的高位地址的位数。可以配置为 0~8 之间的值。
    // 0 是一个特殊的配置值，意味着不启用虚地址缩减模式。
    // 如果配置的值大于 8，则处理器行为不确定
    fn set_val(&mut self, val: u32) -> &mut Self {
        self.bits = val;
        self
    }
}

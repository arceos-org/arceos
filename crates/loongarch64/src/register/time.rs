use super::super::cpu::CPUCFG;
use core::arch::asm;
pub struct Time {}

impl Time {
    pub fn read() -> usize {
        let mut counter: usize;
        unsafe {
            asm!(
            "rdtime.d {},{}",
            out(reg)counter,
            out(reg)_,
            );
        }
        counter
    }
}

pub fn get_timer_freq() -> usize {
    // 获取时钟晶振频率
    // 配置信息字index:4
    let base_freq = CPUCFG::read(4).get_bits(0, 31);
    // 获取时钟倍频因子
    // 配置信息字index:5 位:0-15
    let mul = CPUCFG::read(5).get_bits(0, 15);
    let div = CPUCFG::read(5).get_bits(16, 31);
    // 计算时钟频率
    let cc_freq = base_freq * mul / div;
    cc_freq
}

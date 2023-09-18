/* extioi registers */
use core::arch::asm;

pub const LOONGARCH_IOCSR_EXTIOI_EN_BASE: usize = 0x1600; //扩展 IO 中断[63:0]的中断使能配置
pub const LOONGARCH_IOCSR_EXTIOI_ISR_BASE: usize = 0x1800; //路由至处理器核 0 的扩展 IO 中断[63:0]的中断状态
pub const LOONGARCH_IOCSR_EXTIOI_MAP_BASE: usize = 0x14c0; //EXT_IOI[31:0]的引脚路由方式
pub const LOONGARCH_IOCSR_EXTIOI_ROUTE_BASE: usize = 0x1c00; //EXT_IOI[0]的处理器核路由方式
pub const LOONGARCH_IOCSR_EXRIOI_NODETYPE_BASE: usize = 0x14a0; //16 个结点的映射向量类型 0（软件配置
pub const LOONGARCH_IOCSR_EXRIOI_SEND: usize = 0x1140; // 配置寄存器中增加了一个扩展 IO 中断触发寄存
                                                       // 器，用于将对应的 IO 中断置位

/// 4
pub fn iocsr_write_w(reg: usize, value: u32) {
    unsafe {
        asm!("iocsrwr.w {},{}", in(reg) value, in(reg) reg);
    }
}
// 8
pub fn iocsr_write_d(reg: usize, value: u64) {
    unsafe {
        asm!("iocsrwr.d {},{}", in(reg) value, in(reg) reg);
    }
}
// 2
pub fn iocsr_write_h(reg: usize, value: u16) {
    unsafe {
        asm!("iocsrwr.h {},{}", in(reg) value, in(reg) reg);
    }
}
// 1
pub fn iocsr_write_b(reg: usize, value: u8) {
    unsafe {
        asm!("iocsrwr.b {},{}", in(reg) value, in(reg) reg);
    }
}

pub fn iocsr_read_b(reg: usize) -> u8 {
    let val: u8;
    unsafe {
        asm!("iocsrrd.b {},{}",out(reg) val, in(reg) reg);
    }
    val
}

// 4
pub fn iocsr_read_w(reg: usize) -> u32 {
    let val: u32;
    unsafe {
        asm!("iocsrrd.w {},{}",out(reg) val, in(reg) reg);
    }
    val
}

// 8
pub fn iocsr_read_d(reg: usize) -> u64 {
    let val: u64;
    unsafe {
        asm!("iocsrrd.d {},{}",out(reg) val, in(reg) reg);
    }
    val
}

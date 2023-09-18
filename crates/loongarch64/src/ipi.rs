use crate::consts::*;
use bit_field::BitField;
use core::arch::asm;
pub fn iocsr_write_u32(addr: usize, value: u32) {
    unsafe {
        asm!("iocsrwr.w {},{}", in(reg) value,in(reg) addr);
    }
}

pub fn iocsr_read_u32(addr: usize) -> u32 {
    let mut value: u32;
    unsafe {
        asm!("iocsrrd.w {},{}", out(reg) value, in(reg) addr);
    }
    value
}

pub fn iocsr_write_u64(addr: usize, value: u64) {
    unsafe {
        asm!("iocsrwr.d {},{}", in(reg) value, in(reg) addr);
    }
}

pub fn iocsr_read_u64(addr: usize) -> u64 {
    let mut value: u64;
    unsafe {
        asm!("iocsrrd.d {},{}", out(reg) value, in(reg) addr);
    }
    value
}

fn iocsr_mbuf_send_box_lo(box_: usize) -> usize {
    box_ << 1
}
fn iocsr_mbuf_send_box_hi(box_: usize) -> usize {
    (box_ << 1) + 1
}

pub fn csr_mail_send(entry: u64, cpu: usize, mailbox: usize) {
    let mut val: u64;
    val = IOCSR_MBUF_SEND_BLOCKING;
    val |= (iocsr_mbuf_send_box_hi(mailbox) << IOCSR_MBUF_SEND_BOX_SHIFT) as u64;
    val |= (cpu << IOCSR_MBUF_SEND_CPU_SHIFT) as u64;
    val |= entry & IOCSR_MBUF_SEND_H32_MASK as u64;
    iocsr_write_u64(LOONGARCH_IOCSR_MBUF_SEND, val);
    val = IOCSR_MBUF_SEND_BLOCKING;
    val |= (iocsr_mbuf_send_box_lo(mailbox) << IOCSR_MBUF_SEND_BOX_SHIFT) as u64;
    val |= (cpu << IOCSR_MBUF_SEND_CPU_SHIFT) as u64;
    val |= entry << IOCSR_MBUF_SEND_BUF_SHIFT;
    iocsr_write_u64(LOONGARCH_IOCSR_MBUF_SEND, val);
}

/// IPI_Send 0x1040 WO 32 位中断分发寄存器
/// `[31]` 等待完成标志，置 1 时会等待中断生效
///
/// `[30:26]` 保留
///
/// `[25:16]` 处理器核号
///
/// `[15:5]` 保留
///
/// `[4:0]` 中断向量号，对应 IPI_Status 中的向量
pub fn ipi_write_action(cpu: usize, action: u32) {
    let mut val: u32 = IOCSR_IPI_SEND_BLOCKING;
    for i in 0..32 {
        if action.get_bit(i) {
            val |= (cpu << IOCSR_IPI_SEND_CPU_SHIFT) as u32;
            val |= i as u32;
            iocsr_write_u32(LOONGARCH_IOCSR_IPI_SEND, val);
        }
    }
}

pub fn send_ipi_single(cpu: usize, action: u32) {
    ipi_write_action(cpu, action);
}

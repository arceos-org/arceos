/// LS7A桥片配置空间
/// LS7A桥片配置空间--中断控制器起始地址
/// 0x1000_0000~0x1000_0fff 4k
pub const LS7A_PCH_REG_BASE: usize = 0x1000_0000;
pub const LS7A_MISC_REG_BASE: usize = LS7A_PCH_REG_BASE + 0x00080000;
pub const LS7A_ACPI_REG_BASE: usize = LS7A_MISC_REG_BASE + 0x00050000;
pub const LS7A_RTC_REG_BASE: usize = LS7A_MISC_REG_BASE + 0x00050100;

pub const UART0_IRQ: usize = 2;
pub const KEYBOARD_IRQ: usize = 3;
pub const MOUSE_IRQ: usize = 4;

// 8042 Keyboard Controller
pub const LS7A_I8042_DATA: usize = 0x1fe00060;
pub const LS7A_I8042_COMMAND: usize = 0x1fe00064;
pub const LS7A_I8042_STATUS: usize = 0x1fe00064;

pub const LS7A_INT_MASK_REG: usize = LS7A_PCH_REG_BASE + 0x020; //中断掩码寄存器低32位
pub const LS7A_INT_EDGE_REG: usize = LS7A_PCH_REG_BASE + 0x060; //触发方式寄存器
pub const LS7A_INT_CLEAR_REG: usize = LS7A_PCH_REG_BASE + 0x080; //边沿触发中断清除寄存器
pub const LS7A_INT_HTMSI_VEC_REG: usize = LS7A_PCH_REG_BASE + 0x200; //HT 中断向量寄存器[ 7- 0]
pub const LS7A_INT_STATUS_REG: usize = LS7A_PCH_REG_BASE + 0x3a0; //中断状态（在服务）寄存器 ISR
pub const LS7A_INT_POL_REG: usize = LS7A_PCH_REG_BASE + 0x3e0; //中断触发电平选择寄存器

pub fn ls7a_read_w(addr: usize) -> u32 {
    unsafe { (addr as *const u32).read_volatile() }
}

pub fn ls7a_write_w(addr: usize, value: u32) {
    unsafe {
        (addr as *mut u32).write_volatile(value);
    }
}
pub fn ls7a_write_b(addr: usize, value: u8) {
    unsafe {
        (addr as *mut u8).write_volatile(value);
    }
}
pub fn ls7a_read_b(addr: usize) -> u8 {
    unsafe { (addr as *const u8).read_volatile() }
}

pub fn ls7a_read_d(addr: usize) -> u64 {
    unsafe { (addr as *const u64).read_volatile() }
}

pub fn ls7a_write_d(addr: usize, value: u64) {
    unsafe {
        (addr as *mut u64).write_volatile(value);
    }
}

/// 初始化ls7a中断控制器
pub fn ls7a_intc_init() {
    // enable uart0/keyboard/mouse
    // 使能设备的中断
    ls7a_write_w(
        LS7A_INT_MASK_REG,
        !((0x1 << UART0_IRQ) | (0x1 << KEYBOARD_IRQ) | (0x1 << MOUSE_IRQ)),
    );
    // 触发方式设置寄存器
    // 0：电平触发中断
    // 1：边沿触发中断
    // 这里设置为电平触发
    ls7a_write_w(
        LS7A_INT_EDGE_REG,
        0x1 << (UART0_IRQ | KEYBOARD_IRQ | MOUSE_IRQ),
    );
    // route to the same irq in extioi, pch_irq == extioi_irq
    ls7a_write_b(LS7A_INT_HTMSI_VEC_REG + UART0_IRQ, UART0_IRQ as u8);
    ls7a_write_b(LS7A_INT_HTMSI_VEC_REG + KEYBOARD_IRQ, KEYBOARD_IRQ as u8);
    ls7a_write_b(LS7A_INT_HTMSI_VEC_REG + MOUSE_IRQ, MOUSE_IRQ as u8);
    // 设置中断电平触发极性
    // 对于电平触发类型：
    // 0：高电平触发；
    // 1：低电平触发
    // 这里是高电平触发
    ls7a_write_w(LS7A_INT_POL_REG, 0x0);
}

pub fn ls7a_intc_complete(irq: u64) {
    // 将对应位写1 清除中断
    ls7a_write_d(LS7A_INT_CLEAR_REG, irq);
}

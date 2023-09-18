use super::loongson::{
    iocsr_read_b, iocsr_read_d, iocsr_read_w, iocsr_write_b, iocsr_write_d, iocsr_write_h,
    iocsr_write_w, LOONGARCH_IOCSR_EXRIOI_NODETYPE_BASE, LOONGARCH_IOCSR_EXTIOI_EN_BASE,
    LOONGARCH_IOCSR_EXTIOI_ISR_BASE, LOONGARCH_IOCSR_EXTIOI_MAP_BASE,
    LOONGARCH_IOCSR_EXTIOI_ROUTE_BASE,
};
use super::ls7a::{KEYBOARD_IRQ, MOUSE_IRQ, UART0_IRQ};
use super::register::csr::Register;
use super::register::estat::Estat;
use bit_field::BitField;
use log::{debug, info};
/// 初始化外部中断
pub fn extioi_init() {
    let estat = Estat::read();
    debug!("before_extioi_init_estat={:#x?}", estat.get_val());
    /* let mut enable = 0;
    enable
        .set_bit(KEYBOARD_IRQ, true)
        .set_bit(MOUSE_IRQ, true)
        .set_bit(UART0_IRQ, true);
    info!("extioi_init: enable = {:#b}", enable);*/
    // 使能外部设备中断
    // iocsr_write_d(LOONGARCH_IOCSR_EXTIOI_EN_BASE, enable);

    // extioi[31:0] map to cpu irq pin INT1, other to INT0
    //路由到INT1上
    iocsr_write_b(LOONGARCH_IOCSR_EXTIOI_MAP_BASE, 0x1);
    // extioi IRQ 0-7 route to core 0, use node type 0
    //路由到EXT_IOI_node_type0指向的0号处理器上
    iocsr_write_w(LOONGARCH_IOCSR_EXTIOI_ROUTE_BASE, 0x0);
    // nodetype0 set to 1, always trigger at node 0 */
    //固定分发模式时,只在0号处理器上触发
    iocsr_write_h(LOONGARCH_IOCSR_EXRIOI_NODETYPE_BASE, 0x1);

    //检查扩展i/o触发器是不是全0，即没有被触发的中断
    let extioi_isr = iocsr_read_b(LOONGARCH_IOCSR_EXTIOI_ISR_BASE);
    debug!("extioi_init: extioi_isr = {:#b}", extioi_isr);
    let current_trigger = extioi_claim();
    debug!("extioi_init: current_trigger = {:#b}", current_trigger);
    assert_eq!(extioi_isr, 0);
    let estat = Estat::read();
    debug!("after_extioi_init_estat={:#x?}", estat.get_val());
    debug!("extioi_init: current_trigger = {:#b}", current_trigger);
}

// ask the extioi what interrupt we should serve.
pub fn extioi_claim() -> u64 {
    iocsr_read_d(LOONGARCH_IOCSR_EXTIOI_ISR_BASE)
}

pub fn extioi_complete(irq: u64) {
    iocsr_write_d(LOONGARCH_IOCSR_EXTIOI_ISR_BASE, irq);
}

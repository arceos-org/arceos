use crate::{irq::IrqHandler, mem::phys_to_virt};
use arm_gic::{translate_irq, GenericArmGic, IntId, InterruptType};
use memory_addr::PhysAddr;
use spinlock::SpinNoIrq;

/// The maximum number of IRQs.
pub const MAX_IRQ_COUNT: usize = IntId::GIC_MAX_IRQ;
/// The timer IRQ number.
pub const TIMER_IRQ_NUM: usize = translate_irq(14, InterruptType::PPI).unwrap();

/// The UART IRQ number.
pub const UART_IRQ_NUM: usize = translate_irq(axconfig::UART_IRQ, InterruptType::SPI).unwrap();

const GICD_BASE: PhysAddr = PhysAddr::from(axconfig::GICD_PADDR);
const GICC_BASE: PhysAddr = PhysAddr::from(axconfig::GICC_PADDR);

cfg_if::cfg_if! {
    if #[cfg(platform_family= "aarch64-rk3588j")] {
        use arm_gic::GicV3;
        static mut GIC: SpinNoIrq<GicV3> =
            SpinNoIrq::new(GicV3::new(phys_to_virt(GICD_BASE).as_mut_ptr(), phys_to_virt(GICC_BASE).as_mut_ptr()));
    } else {
        use arm_gic::GicV2;
        static mut GIC: SpinNoIrq<GicV2> =
            SpinNoIrq::new(GicV2::new(phys_to_virt(GICD_BASE).as_mut_ptr(), phys_to_virt(GICC_BASE).as_mut_ptr()));
    }
}

/// Enables or disables the given IRQ.
pub fn set_enable(irq_num: usize, enabled: bool) {
    trace!("GICD set enable: {} {}", irq_num, enabled);

    // SAFETY:
    // access percpu interface through get_mut, no need to lock
    // it will introduce side effects: need to add unsafe
    // Acceptable compared to data competition
    unsafe {
        if enabled {
            GIC.lock().enable_interrupt(irq_num.into());
        } else {
            GIC.lock().disable_interrupt(irq_num.into());
        }
    }
}

/// Registers an IRQ handler for the given IRQ.
///
/// It also enables the IRQ if the registration succeeds. It returns `false` if
/// the registration failed.
pub fn register_handler(irq_num: usize, handler: IrqHandler) -> bool {
    trace!("register handler irq {}", irq_num);
    crate::irq::register_handler_common(irq_num, handler)
}

/// Dispatches the IRQ.
///
/// This function is called by the common interrupt handler. It looks
/// up in the IRQ handler table and calls the corresponding handler. If
/// necessary, it also acknowledges the interrupt controller after handling.
pub fn dispatch_irq(_unused: usize) {
    // actually no need to lock
    let intid = unsafe { GIC.get_mut().get_and_acknowledge_interrupt() };
    if let Some(id) = intid {
        crate::irq::dispatch_irq_common(id.into());
        unsafe {
            GIC.get_mut().end_interrupt(id);
        }
    }
}

/// Initializes GICD, GICC on the primary CPU.
pub(crate) fn init_primary() {
    info!("Initialize GICv2...");
    unsafe { GIC.lock().init_primary() };
}

/// Initializes GICC on secondary CPUs.
#[cfg(feature = "smp")]
pub(crate) fn init_secondary() {
    // per cpu handle, no need lock
    unsafe { GIC.get_mut().per_cpu_init() };
}

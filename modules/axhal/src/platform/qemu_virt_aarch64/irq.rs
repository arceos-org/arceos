use crate::irq::IrqHandler;
use crate::mem::phys_to_virt;
use arm_gic::gic_v2::{GicCpuInterface, GicDistributor};
use memory_addr::PhysAddr;
use spinlock::SpinNoIrq;

pub const MAX_IRQ_COUNT: usize = 1024;

const GIC_BASE: usize = 0x0800_0000;
const GICD_BASE: PhysAddr = PhysAddr::from(GIC_BASE);
const GICC_BASE: PhysAddr = PhysAddr::from(GIC_BASE + 0x10000);

static GICD: SpinNoIrq<GicDistributor> =
    SpinNoIrq::new(GicDistributor::new(phys_to_virt(GICD_BASE).as_mut_ptr()));

// per-CPU, no lock
static GICC: GicCpuInterface = GicCpuInterface::new(phys_to_virt(GICC_BASE).as_mut_ptr());

pub fn set_enable(irq_num: usize, enabled: bool) {
    GICD.lock().set_enable(irq_num as _, enabled);
}

pub fn register_handler(irq_num: usize, handler: IrqHandler) -> bool {
    crate::irq::register_handler_common(irq_num, handler)
}

/// Platform-dependent IRQ handler
pub(crate) fn platform_handle_irq(_unused: usize) {
    GICC.handle_irq(|irq_num| crate::irq::dispatch_irq(irq_num as _));
    crate::trap::task_try_preempt();
}

pub(super) fn init() {
    GICD.lock().init();
}

pub(super) fn init_percpu(_cpu_id: usize) {
    GICC.init();
}

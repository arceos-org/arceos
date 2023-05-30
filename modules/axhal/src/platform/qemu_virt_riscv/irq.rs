//! TODO: PLIC

use crate::{irq::IrqHandler, mem::phys_to_virt};
use lazy_init::LazyInit;
use memory_addr::PhysAddr;
use plic::Plic;
use riscv::register::sie;
use spinlock::SpinNoIrq;

const PLIC_BASE: PhysAddr = PhysAddr::from(0x0c00_0000);

static PLIC: SpinNoIrq<Plic> =
    SpinNoIrq::new(Plic::new(phys_to_virt(PLIC_BASE).as_usize() as *mut u8));

/// `Interrupt` bit in `scause`
pub(super) const INTC_IRQ_BASE: usize = 1 << (usize::BITS - 1);

/// Supervisor software interrupt in `scause`
#[allow(unused)]
pub(super) const S_SOFT: usize = INTC_IRQ_BASE + 1;

/// Supervisor timer interrupt in `scause`
pub(super) const S_TIMER: usize = INTC_IRQ_BASE + 5;

/// Supervisor external interrupt in `scause`
pub(super) const S_EXT: usize = INTC_IRQ_BASE + 9;

static TIMER_HANDLER: LazyInit<IrqHandler> = LazyInit::new();

/// The maximum number of IRQs.
pub const MAX_IRQ_COUNT: usize = 1024;

/// The timer IRQ number (supervisor timer interrupt in `scause`).
pub const TIMER_IRQ_NUM: usize = S_TIMER;

macro_rules! with_cause {
    ($cause: expr, @TIMER => $timer_op: expr, @EXT => $ext_op: expr $(,)?) => {
        match $cause {
            S_TIMER => $timer_op,
            S_EXT => $ext_op,
            _ => panic!("invalid trap cause: {:#x}", $cause),
        }
    };
}

/// Enables or disables the given IRQ.
pub fn set_enable(irq_num: usize, _enabled: bool) {
    let hart_id = crate::cpu::this_cpu_id();
    info!("hart_id: {:#x}", hart_id);
    PLIC.lock().enable(hart_id, irq_num);
    PLIC.lock().set_priority(irq_num, 1);
}

/// Registers an IRQ handler for the given IRQ.
///
/// It also enables the IRQ if the registration succeeds. It returns `false` if
/// the registration failed.
pub fn register_handler(irq_num: usize, handler: IrqHandler) -> bool {
    // with_cause!(
    //     scause,
    //     @TIMER => if !TIMER_HANDLER.is_init() {
    //         TIMER_HANDLER.init_by(handler);
    //         true
    //     } else {
    //         false
    //     },
    //     @EXT => crate::irq::register_handler_common(scause & !INTC_IRQ_BASE, handler),
    // )
    trace!("irq num: {:#x}", irq_num);
    match irq_num {
        S_TIMER => {
            if !TIMER_HANDLER.is_init() {
                TIMER_HANDLER.init_by(handler);
                true
            } else {
                false
            }
        }
        _ => crate::irq::register_handler_common(irq_num, handler),
    }
}

/// Dispatches the IRQ.
///
/// This function is called by the common interrupt handler. It looks
/// up in the IRQ handler table and calls the corresponding handler. If
/// necessary, it also acknowledges the interrupt controller after handling.
pub fn dispatch_irq(scause: usize) {
    with_cause!(
        scause,
        @TIMER => {
            trace!("IRQ: timer");
            TIMER_HANDLER();
        },
        @EXT =>
        {
            // TODO: get IRQ number from PLIC
            let hart_id = crate::cpu::this_cpu_id();
            let irq_num = PLIC.lock().claim(hart_id);
            debug!("External IRQ: {:#x}", irq_num);
            crate::irq::dispatch_irq_common(irq_num as usize);
            PLIC.lock().complete(hart_id, irq_num);
        }
    );
}

pub(super) fn init_percpu() {
    // enable soft interrupts, timer interrupts, and external interrupts
    unsafe {
        sie::set_ssoft();
        sie::set_stimer();
        sie::set_sext();
    }

    let hart_id = crate::cpu::this_cpu_id();
    PLIC.lock().set_threshold(hart_id, 1, 0);
    PLIC.lock().set_threshold(hart_id, 0, 1);
}

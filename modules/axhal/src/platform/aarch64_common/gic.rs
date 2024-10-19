use crate::{irq::IrqHandler, mem::phys_to_virt};
use arm_gicv2::{GicCpuInterface, GicDistributor, InterruptType, translate_irq};
use axconfig::devices::{GICC_PADDR, GICD_PADDR, UART_IRQ};
use kspin::SpinNoIrq;
use memory_addr::PhysAddr;

/// The maximum number of IRQs.
pub const MAX_IRQ_COUNT: usize = 1024;

#[cfg(not(feature = "hv"))]
/// The timer IRQ number.
pub const TIMER_IRQ_NUM: usize = translate_irq(14, InterruptType::PPI).unwrap();

#[cfg(feature = "hv")]
/// Non-secure EL2 Physical Timer irq number.
pub const TIMER_IRQ_NUM: usize = translate_irq(10, InterruptType::PPI).unwrap();

/// The UART IRQ number.
pub const UART_IRQ_NUM: usize = translate_irq(UART_IRQ, InterruptType::SPI).unwrap();

const GICD_BASE: PhysAddr = pa!(GICD_PADDR);
const GICC_BASE: PhysAddr = pa!(GICC_PADDR);

static GICD: SpinNoIrq<GicDistributor> =
    SpinNoIrq::new(GicDistributor::new(phys_to_virt(GICD_BASE).as_mut_ptr()));

// per-CPU, no lock
static GICC: GicCpuInterface = GicCpuInterface::new(phys_to_virt(GICC_BASE).as_mut_ptr());

/// Enables or disables the given IRQ.
pub fn set_enable(irq_num: usize, enabled: bool) {
    trace!("GICD set enable: {} {}", irq_num, enabled);
    GICD.lock().set_enable(irq_num as _, enabled);
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
    GICC.handle_irq(|irq_num| crate::irq::dispatch_irq_common(irq_num as _));
}

/// Initializes GICD, GICC on the primary CPU.
pub(crate) fn init_primary() {
    info!("Initialize GICv2...");
    GICD.lock().init();
    GICC.init();
}

/// Initializes GICC on secondary CPUs.
#[cfg(feature = "smp")]
pub(crate) fn init_secondary() {
    GICC.init();
}

#[cfg(feature = "hv")]
mod gic_if {

    use super::{GICC, GICD};
    use arm_gicv2::GicTrait;
    struct GicIfImpl;

    #[crate_interface::impl_interface]
    impl GicTrait for GicIfImpl {
        /// Sets the enable status of a specific interrupt in the GICD (Distributor).
        ///
        /// # Parameters
        /// - `vector`: The interrupt vector number, identifying the interrupt to be enabled or disabled.
        /// - `enable`: A boolean value indicating whether to enable the interrupt. `true` enables the interrupt, `false` disables it.
        ///
        /// This function locks and accesses the GICD controller, then sets the enable status of the specified interrupt vector based on the `enable` parameter.
        /// It provides a mechanism for controlling whether interrupts can trigger CPU responses, used for interrupt management.
        fn set_enable(vector: usize, enable: bool) {
            GICD.lock().set_enable(vector, enable);
        }

        /// Retrieves the enable status of a specified interrupt vector from the GICD.
        ///
        /// # Parameters
        /// - `vector`: The index of the interrupt vector, used to identify a specific interrupt source.
        ///
        /// # Returns
        /// - `bool`: Indicates whether the specified interrupt vector is enabled. `true` means the interrupt vector is enabled, `false` means it is not enabled.
        fn get_enable(vector: usize) -> bool {
            GICD.lock().get_enable(vector)
        }

        /// Get the type of the GICD register
        ///
        /// This function locks the GICD and calls its internal `get_typer` method to retrieve the type of the GICD register
        ///
        /// # Returns
        /// * `u32` - The type of the GICD register
        fn get_typer() -> u32 {
            GICD.lock().get_typer()
        }

        /// Get the Implementer ID Register (IIDR) of the interrupt controller
        ///
        /// This function locks the GICD (interrupt controller) and calls its `get_iidr` method to retrieve the value of the Implementer ID Register.
        /// This register can be used to identify the specific hardware implementer and version.
        fn get_iidr() -> u32 {
            GICD.lock().get_iidr()
        }

        /// Set the state of an interrupt source
        ///
        /// This function updates the state of a specific interrupt source in the GICD (Interrupt Controller).
        /// It first locks the GICD and then updates the interrupt source state using the provided interrupt ID (`int_id`),
        /// new state value (`state`), and current CPU ID (`current_cpu_id`).
        ///
        /// Parameters:
        /// - `int_id`: The ID of the interrupt source.
        /// - `state`: The new state value for the interrupt source.
        /// - `current_cpu_id`: The ID of the current CPU.
        fn set_state(int_id: usize, state: usize, current_cpu_id: usize) {
            GICD.lock().set_state(int_id, state, current_cpu_id);
        }

        /// Get the state of an interrupt source
        ///
        /// This function retrieves the current state of a specific interrupt source.
        ///
        /// Parameters:
        /// - `int_id`: The ID of the interrupt source.
        ///
        /// Returns:
        /// - The current state value.
        fn get_state(int_id: usize) -> usize {
            GICD.lock().get_state(int_id)
        }

        /// Set the ICFGR (Interrupt Configuration and Control Register)
        ///
        /// This function sets the configuration of a specific interrupt source in the ICFGR.
        ///
        /// Parameters:
        /// - `int_id`: The ID of the interrupt source.
        /// - `cfg`: The new configuration value.
        fn set_icfgr(int_id: usize, cfg: u8) {
            GICD.lock().set_icfgr(int_id, cfg);
        }

        /// Get the target CPU for an interrupt source
        ///
        /// This function retrieves the target CPU for a specific interrupt source.
        ///
        /// Parameters:
        /// - `int_id`: The ID of the interrupt source.
        ///
        /// Returns:
        /// - The target CPU ID.
        fn get_target_cpu(int_id: usize) -> usize {
            GICD.lock().get_target_cpu(int_id)
        }

        /// Set the target CPU for an interrupt source
        ///
        /// This function sets the target CPU for a specific interrupt source.
        ///
        /// Parameters:
        /// - `int_id`: The ID of the interrupt source.
        /// - `target`: The new target CPU value.
        fn set_target_cpu(int_id: usize, target: u8) {
            GICD.lock().set_target_cpu(int_id, target);
        }

        /// Get the priority of an interrupt source
        ///
        /// This function retrieves the priority of a specific interrupt source.
        ///
        /// Parameters:
        /// - `int_id`: The ID of the interrupt source.
        ///
        /// Returns:
        /// - The priority value.
        fn get_priority(int_id: usize) -> usize {
            GICD.lock().get_priority(int_id)
        }

        /// Set the priority of an interrupt source
        ///
        /// This function sets the priority of a specific interrupt source.
        ///
        /// Parameters:
        /// - `int_id`: The ID of the interrupt source.
        /// - `priority`: The new priority value.
        fn set_priority(int_id: usize, priority: u8) {
            GICD.lock().set_priority(int_id, priority);
        }
    }
}

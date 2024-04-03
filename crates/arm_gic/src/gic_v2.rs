//! Types and definitions for GICv2.
//!
//! The official documentation: <https://developer.arm.com/documentation/ihi0048/latest/>

use core::ptr::NonNull;

use crate::registers::gicv2_regs::*;

use crate::{GenericArmGic, IntId, TriggerMode};
use tock_registers::interfaces::{Readable, Writeable};

/// The GIC distributor.
///
/// The Distributor block performs interrupt prioritization and distribution
/// to the CPU interface blocks that connect to the processors in the system.
///
/// The Distributor provides a programming interface for:
/// - Globally enabling the forwarding of interrupts to the CPU interfaces.
/// - Enabling or disabling each interrupt.
/// - Setting the priority level of each interrupt.
/// - Setting the target processor list of each interrupt.
/// - Setting each peripheral interrupt to be level-sensitive or edge-triggered.
/// - Setting each interrupt as either Group 0 or Group 1.
/// - Forwarding an SGI to one or more target processors.
///
/// In addition, the Distributor provides:
/// - visibility of the state of each interrupt
/// - a mechanism for software to set or clear the pending state of a peripheral
///   interrupt.
#[derive(Debug, Copy, Clone)]
struct GicDistributor {
    base: NonNull<GicDistributorRegs>,
    support_irqs: usize,
    #[allow(dead_code)]
    support_cpu: usize,
}

impl GicDistributor {
    const GICD_DISABLE: u32 = 0;
    const GICD_ENABLE: u32 = 1;

    const CPU_NUM_SHIFT: usize = 5;
    const CPU_NUM_MASK: u32 = 0b111;
    const IT_LINES_NUM_MASK: u32 = 0b11111;

    /// Construct a new GIC distributor instance from the base address.
    pub const fn new(base: *mut u8) -> Self {
        Self {
            base: NonNull::new(base).unwrap().cast(),
            support_irqs: 0,
            support_cpu: 0,
        }
    }

    const fn regs(&self) -> &GicDistributorRegs {
        unsafe { self.base.as_ref() }
    }

    /// Configures the trigger type for the interrupt with the given ID.
    fn set_trigger(&mut self, id: usize, tm: TriggerMode) {
        // type is encoded with two bits, MSB of the two determine type
        // 16 irqs encoded per ICFGR register
        let index = id >> 4;
        let bit_shift = ((id & 0xf) << 1) + 1;

        let mut reg_val = self.regs().ICFGR[index].get();
        match tm {
            TriggerMode::Edge => reg_val |= 1 << bit_shift,
            TriggerMode::Level => reg_val &= !(1 << bit_shift),
        }

        self.regs().ICFGR[index].set(reg_val);
    }

    /// Initializes the GIC distributor.
    ///
    /// It disables all interrupts, sets the target of all SPIs to CPU 0,
    /// configures all SPIs to be edge-triggered, and finally enables the GICD.
    ///
    /// This function should be called only once.
    pub fn init(&mut self) {
        let typer = self.regs().TYPER.get();

        // The maximum number of interrupts that the GIC supports
        // If ITLinesNumber=N, the maximum number of interrupts is 32(N+1)
        let irq_num = (((typer & Self::IT_LINES_NUM_MASK) + 1) * 32) as usize;
        match irq_num {
            0..=IntId::GIC_MAX_IRQ => self.support_irqs = irq_num,
            _ => self.support_irqs = IntId::GIC_MAX_IRQ,
        }

        self.support_cpu = (((typer >> Self::CPU_NUM_SHIFT) & Self::CPU_NUM_MASK) + 1) as usize;

        // disable GICD
        self.regs().CTLR.set(Self::GICD_DISABLE);

        // Set all global interrupts to CPU0.
        for i in (IntId::SPI_START..self.support_irqs).step_by(4) {
            // Set external interrupts to target cpu 0
            // once time set 4 interrupts
            self.regs().ITARGETSR[i / 4].set(0x01_01_01_01);
        }

        // Initialize all the SPIs to edge triggered
        for i in IntId::SPI_START..self.support_irqs {
            self.set_trigger(i, TriggerMode::Edge);
        }

        // Set priority on all global interrupts
        for i in (IntId::SPI_START..self.support_irqs).step_by(4) {
            // once time set 4 interrupts
            self.regs().IPRIORITYR[i / 4].set(0xa0_a0_a0_a0);
        }

        // Deactivate and disable all SPIs
        for i in (IntId::SPI_START..self.support_irqs).step_by(32) {
            self.regs().ICACTIVER[i / 32].set(u32::MAX);
            self.regs().ICENABLER[i / 32].set(u32::MAX);
        }

        // enable GIC0
        self.regs().CTLR.set(Self::GICD_ENABLE);
    }
}

/// The GIC CPU interface.
///
/// Each CPU interface block performs priority masking and preemption
/// handling for a connected processor in the system.
///
/// Each CPU interface provides a programming interface for:
///
/// - enabling the signaling of interrupt requests to the processor
/// - acknowledging an interrupt
/// - indicating completion of the processing of an interrupt
/// - setting an interrupt priority mask for the processor
/// - defining the preemption policy for the processor
/// - determining the highest priority pending interrupt for the processor.
#[derive(Debug, Copy, Clone)]
struct GicCpuInterface {
    base: NonNull<GicCpuInterfaceRegs>,
}

impl GicCpuInterface {
    const GICC_ENABLE: u32 = 1;

    /// Construct a new GIC CPU interface instance from the base address.
    pub const fn new(base: *mut u8) -> Self {
        Self {
            base: NonNull::new(base).unwrap().cast(),
        }
    }

    const fn regs(&self) -> &GicCpuInterfaceRegs {
        unsafe { self.base.as_ref() }
    }

    /// Initializes the GIC CPU interface.
    ///
    /// It unmask interrupts at all priority levels and enables the GICC.
    ///
    /// This function should be called only once.
    pub fn init(&self, gicd: &GicDistributor) {
        // Deactivate and disable all private interrupts
        gicd.regs().ICACTIVER[0].set(u32::MAX);
        gicd.regs().ICENABLER[0].set(u32::MAX);

        // Set priority on private interrupts
        for i in (0..IntId::SPI_START).step_by(4) {
            // once time set 4 interrupts
            gicd.regs().IPRIORITYR[i / 4].set(0xa0_a0_a0_a0);
        }

        // unmask interrupts at all priority levels
        self.regs().PMR.set(0xff);
        // enable GIC0
        self.regs().CTLR.set(Self::GICC_ENABLE);
    }
}

unsafe impl Send for GicDistributor {}
unsafe impl Sync for GicDistributor {}

unsafe impl Send for GicCpuInterface {}
unsafe impl Sync for GicCpuInterface {}

/// Driver for an Arm Generic Interrupt Controller version 2.
#[derive(Debug, Copy, Clone)]
pub struct GicV2 {
    gicd: GicDistributor,
    gicc: GicCpuInterface,
}

unsafe impl Send for GicV2 {}
unsafe impl Sync for GicV2 {}

impl GicV2 {
    /// # Safety
    ///
    /// The given base addresses must point to the GIC distributor and redistributor registers
    /// respectively. These regions must be mapped into the address space of the process as device
    /// memory, and not have any other aliases, either via another instance of this driver or
    /// otherwise.
    pub const fn new(gicd: *mut u8, gicc: *mut u8) -> Self {
        Self {
            gicd: GicDistributor::new(gicd),
            gicc: GicCpuInterface::new(gicc),
        }
    }
}

impl GenericArmGic for GicV2 {
    /// Initialises the GIC.
    fn init_primary(&mut self) {
        self.gicd.init();
        self.gicc.init(&self.gicd);
    }

    /// Initialises the GIC for the current CPU core.
    fn per_cpu_init(&mut self) {
        self.gicc.init(&self.gicd);
    }

    /// Configures the trigger type for the interrupt with the given ID.
    fn set_trigger(&mut self, intid: IntId, tm: TriggerMode) {
        // Only configurable for SPI interrupts
        if intid.0 < IntId::SPI_START {
            return;
        }
        self.gicd.set_trigger(intid.0, tm);
    }

    /// Enables the interrupt with the given ID.
    fn enable_interrupt(&mut self, intid: IntId) {
        let index = intid.0 / 32;
        let bit = 1 << (intid.0 % 32);
        self.gicd.regs().ISENABLER[index].set(bit);
    }

    /// Disable the interrupt with the given ID.
    fn disable_interrupt(&mut self, intid: IntId) {
        let index = intid.0 / 32;
        let bit = 1 << (intid.0 % 32);
        self.gicd.regs().ICENABLER[index].set(bit);
    }

    fn get_and_acknowledge_interrupt(&self) -> Option<IntId> {
        let iar = self.gicc.regs().IAR.get();
        let id = (iar & 0x3ff) as usize;
        if id >= IntId::SPECIAL_START {
            None
        } else {
            Some(IntId(id))
        }
    }

    /// Informs the interrupt controller that the CPU has completed processing the given interrupt.
    /// This drops the interrupt priority and deactivates the interrupt.
    fn end_interrupt(&self, intid: IntId) {
        self.gicc.regs().EOIR.set(intid.0 as u32);
    }
}

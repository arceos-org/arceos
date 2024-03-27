//! Types and definitions for GICv2.
//!
//! The official documentation: <https://developer.arm.com/documentation/ihi0048/latest/>

use core::ptr::NonNull;
use core::hint::spin_loop;
use aarch64_cpu::registers::MPIDR_EL1;
use tock_registers::interfaces::{Readable, Writeable};

use crate::{TriggerMode, IntId, GenericArmGic};
use crate::registers::gicv3_regs::*;
use crate::sysregs::{read_sysreg, write_sysreg};

const SGI_OFFSET: usize = 0x10000;

/// The GIC-V3 distributor.
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
#[derive(Debug,Copy,Clone)]
pub struct GicDistributor {
    base: NonNull<GicDistributorRegs>,
    support_irqs: usize,
    // extend spi
    support_espi: usize,
    #[allow(dead_code)]
    support_cpu: usize,
}

/// The GIC-V3 redistributor.
#[derive(Debug,Copy,Clone)]
pub struct GicRedistributor {
    gicr_base: NonNull<GicRedistributorRegs>,
    support_ppi: usize,
}

unsafe impl Send for GicDistributor {}
unsafe impl Sync for GicDistributor {}

unsafe impl Send for GicRedistributor {}
unsafe impl Sync for GicRedistributor {}

struct Gicv3Quirk {
    #[allow(dead_code)]
    desc: &'static str,
    iidr: u32,
    mask: u32,
}

impl GicDistributor {
    const GICD_DISABLE: u32 = 0;

    const IT_LINES_NUM_MASK: u32 = 0x1f;
    const ESPI_MASK: u32 = 0b1_0000_0000;
    const ESPI_RANGE_SHIF: u32 = 27;

    const GIC_PIDR2_ARCH_MASK:u32 = 0xf0;
    const GIC_PIDR2_ARCH_GICV3:u32 = 0x30;
    const GIC_PIDR2_ARCH_GICV4:u32 = 0x40;

    const GICD_RWP_MASK: u32 = 1<<31;

    /// The GIC-V3 erratum.
    ///
    /// Gicv3 errata records synchronized from linnux to prevent possible error conditions.
    const GICV3_QUIRKS: [Gicv3Quirk; 4] = [
        Gicv3Quirk {
            desc: "GICv3: HIP06 erratum 161010803",
            iidr: 0x0204043b,
            mask: 0xffffffff,
        },
        Gicv3Quirk {
            desc: "GICv3: HIP07 erratum 161010803",
            iidr: 0x00000000,
            mask: 0xffffffff,
        },
        Gicv3Quirk {
            desc: "GICv3: Cavium erratum 38539",
            iidr: 0xa000034c,
            mask: 0xe8f00fff,
        },
        Gicv3Quirk {
            desc: "GICv3: NVIDIA erratum T241-FABRIC-4",
            iidr: 0x0402043b,
            mask: 0xffffffff,
        },
    ];

    /// Construct a new GIC distributor instance from the base address.
    pub const fn new(base: *mut u8) -> Self {
        Self {
            base: NonNull::new(base).unwrap().cast(),
            support_cpu: 0,
            support_espi: 0,
            support_irqs: 0,
        }
    }

    const fn regs(&self) -> &GicDistributorRegs {
        unsafe { self.base.as_ref() }
    }

    fn validate_dist_version(&self) {
        let pidr2 = self.regs().PIDR2.get() & Self::GIC_PIDR2_ARCH_MASK; 
        match pidr2 {
            Self::GIC_PIDR2_ARCH_GICV3 | Self::GIC_PIDR2_ARCH_GICV4 => (),
            _ => panic!("unvalid gic pidr2")
        }
    }

    fn check_gic_erratum(&self) {
        let iidr = self.regs().IIDR.get(); 
        for e in Self::GICV3_QUIRKS {
            if iidr & e.mask == e.iidr {
                panic!("gic need fix erratum")
            }
        }
    }

    fn init_check(&self) {
        self.validate_dist_version();
        self.check_gic_erratum();
    }

    fn base_init(&mut self) {
        let typer = self.regs().TYPER.get();

        // The maximum number of interrupts that the GIC supports
        //. If ITLinesNumber=N, the maximum number of interrupts is 32(N+1)
        let irq_num = (((typer & Self::IT_LINES_NUM_MASK) + 1) * 32) as usize;
        match irq_num {
            0..=IntId::GIC_MAX_IRQ => self.support_irqs = irq_num,
            _ => self.support_irqs = IntId::GIC_MAX_IRQ,
        }

        // Extended SPI range uses INTIDs 4096 - 5119.
        // This range of SPIs is not available when the GIC is operating in legacy mode.
        // GICD_TYPER.ESPI indicates whether the extended SPI range is supported or not.
        // Maximum Extended SPI INTID is (32*(ESPI_range + 1) + 4095)
        self.support_espi = match typer & Self::ESPI_MASK == 0 {
            false =>  {
                let espi_range = (typer >> Self::ESPI_RANGE_SHIF) & Self::IT_LINES_NUM_MASK;
                ((espi_range + 1) * 32) as usize
            }
            true => 0,
        }
    }

    fn wait_rwp(&self) {
        let mut loop_count = 10000;
        loop {
            if loop_count == 0 {
                panic!("wait timeout");
            }

            // When RWP is 0b0, no register write in progress
            match self.regs().CTLR.get() & Self::GICD_RWP_MASK != 0 {
                true => {spin_loop();},
                false => break,
            }
            loop_count -= 1;
        }
    }

    fn espi_disable(&self) {
        // disable all espi interrupt
        for i in (0..self.support_espi).step_by(32) {
            self.regs().ICENABLERnE[i / 32].set(u32::MAX);
            self.regs().ICACTIVERnE[i / 32].set(u32::MAX);
        }

        // Configure all ESPI as non-secure Group-1   
        for i in (0..self.support_espi).step_by(32) {
            self.regs().IGROUPRnE[i / 32].set(u32::MAX);
        }
        // Configure all ESPI as level-sensitive
        for i in (0..self.support_espi).step_by(16) {
            self.regs().ICFGRnE[i / 16].set(0);
        }

        // Configure all ESPI as default priority 
        for i in (0..self.support_espi).step_by(4) {
            self.regs().ICFGRnE[i / 4].set(0xa0_a0_a0_a0);
        }
    }

    fn mpidr_affinity_level(mpidr: u64, level: u32) -> u64 {
        match level {
            3 => mpidr >> 32 & 0xff,
            2 => mpidr >> 16 & 0xff,
            1 => mpidr >> 8 & 0xff,
            0 => mpidr & 0xff,
            _ => panic!("invalid affinity level"),
        }
    }

    fn mpidr_to_affinity_level(mpidr: u64) -> u64 {
        Self::mpidr_affinity_level(mpidr,3) << 32 |
            Self::mpidr_affinity_level(mpidr,2) << 16 |
            Self::mpidr_affinity_level(mpidr,1) << 8 |
            Self::mpidr_affinity_level(mpidr,0)
    }

    fn init(&mut self) {
        self.init_check();
        self.base_init();

        // disable GICD
        self.regs().CTLR.set(Self::GICD_DISABLE);
        self.wait_rwp();

        self.espi_disable();

        // Configure all SPIs as non-secure Group-1. This will only matter
        // if the GIC only has a single security state.
        // This will not do the right thing if the kernel is running in
        // secure mode,
        for i in (IntId::SPI_START..self.support_irqs).step_by(32) {
            self.regs().IGROUPR[i / 32].set(u32::MAX);
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

        // Enable affinity routing and group1
        self.regs().CTLR.set((GicdCtlr::ARE_S | GicdCtlr::EnableGrp1NS).bits());
        self.wait_rwp();

        // Set all global interrupts to current cpu.
        let mpidr:u64 = MPIDR_EL1.get() & 0xff00ffffff;
        for i in IntId::SPI_START..self.support_irqs {
            // Set external interrupts to target cpu 0
            self.regs().IROUTER[i].set(Self::mpidr_to_affinity_level(mpidr));
        }

        // set app espi to current cpu
        for i in 0..self.support_espi {
            // Set external interrupts to target cpu 0
            self.regs().IROUTERnE[i].set(Self::mpidr_to_affinity_level(mpidr));
        }
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
}

impl GicRedistributor {
    /// Construct a new GIC Redistributor instance from the base address.
    pub const fn new(base: *mut u8) -> Self {
        Self {
            gicr_base: NonNull::new(base).unwrap().cast(),
            support_ppi: 0,
        }
    }

    const fn gicr_regs(&self) -> &GicRedistributorRegs {
        unsafe { self.gicr_base.as_ref() }
    }

    const fn sgi_regs(&self) -> &GicSgiRegs {
        // SAFETY: Writing to this system register doesn't access memory in any way
        unsafe {
            let gicr_addr = self.gicr_base.as_ptr(); 
            let sgi_base: NonNull<GicSgiRegs> = NonNull::new(gicr_addr.byte_add(SGI_OFFSET)).unwrap().cast();
            sgi_base.as_ref()
        }
    }

    fn redis_enable(&self) {
        let mut waker = self.gicr_regs().WAKER.get();
        // Wake up this CPU redistributor
        waker &= !(WakerFlags::PROCESSOR_SLEEP.bits());
        self.gicr_regs().WAKER.set(waker);
        
        while WakerFlags::from_bits_truncate(self.gicr_regs().WAKER.get()).contains(WakerFlags::CHILDREN_ASLEEP) {
            spin_loop();
        }
    }

    fn base_init(&mut self) {
        let typer = self.gicr_regs().TYPER.get() as usize;
        let mut ppinum = typer >> 27 & 0x1f;
        ppinum = match ppinum {
            0 => 16,
            1|2 =>  16 + 32 * ppinum,
            _ => panic!("invalid ppinum"),
        };

        self.support_ppi = self.support_ppi.min(ppinum);
    }

    fn init(&mut self) {
        self.base_init();
        self.redis_enable();
        // Configure SGIs/PPIs as non-secure Group-1
        for i in (0..self.support_ppi + 16).step_by(32) {
            self.sgi_regs().IGROUPR0[i / 32].set(u32::MAX);
        }
        
        // Deactivate and disable all private interrupts
        for i in (0..self.support_ppi + 16).step_by(32) {
            self.sgi_regs().ICACTIVER[i / 32].set(u32::MAX);
            self.sgi_regs().ICENABLER[i / 32].set(u32::MAX);
        }

        // Set priority on private interrupts
        for i in (0..self.support_ppi + 16).step_by(4) {
            // once time set 4 interrupts
            self.sgi_regs().IPRIORITYR[i / 4].set(0xa0_a0_a0_a0);
        }
    }

    /// Configures the trigger type for the interrupt with the given ID.
    fn set_trigger(&mut self, id: usize, tm: TriggerMode) {
        // type is encoded with two bits, MSB of the two determine type
        // 16 irqs encoded per ICFGR register
        let index = id >> 4;
        let bit_shift = ((id & 0xf) << 1) + 1;

        let mut reg_val = self.sgi_regs().ICFGR[index].get();
        match tm {
            TriggerMode::Edge => reg_val |= 1 << bit_shift,
            TriggerMode::Level => reg_val &= !(1 << bit_shift),
        }

        self.sgi_regs().ICFGR[index].set(reg_val);
    }
}

/// Driver for an Arm Generic Interrupt Controller version 3 (or 4).
#[derive(Debug,Copy,Clone)]
pub struct GicV3 {
    gicd: GicDistributor,
    gicr: GicRedistributor,
}

impl GicV3 {
    /// Constructs a new instance of the driver for a GIC with the given distributor and
    /// redistributor base addresses.
    ///
    /// # Safety
    ///
    /// The given base addresses must point to the GIC distributor and redistributor registers
    /// respectively. These regions must be mapped into the address space of the process as device
    /// memory, and not have any other aliases, either via another instance of this driver or
    /// otherwise.
    pub const fn new(gicd: *mut u8, gicr: *mut u8) -> Self {
        Self {
            gicd: GicDistributor::new(gicd),
            gicr: GicRedistributor::new(gicr),
        }
    }

    fn cpu_sys_reg_init(&mut self) {
        // SAFETY: Writing to this system register doesn't access memory in any way.
        unsafe {
            // Enable system register access.
            write_sysreg!(icc_sre_el1, 0x01);
        }

        unsafe {
            // Enable system register access.
            write_sysreg!(icc_pmr_el1, 0xf0);
        }

        // SAFETY: Writing to this system register doesn't access memory in any way.
        unsafe {
            // Disable use of `ICC_PMR_EL1` as a hint for interrupt distribution, configure a write
            // to an EOI register to also deactivate the interrupt, and configure preemption groups
            // for group 0 and group 1 interrupts separately.
            write_sysreg!(icc_ctlr_el1, 0x00);
        }

        unsafe {
            // Enable non-secure group 1.
            write_sysreg!(icc_igrpen1_el1, 0x00000001);
        }
    }

}

impl GenericArmGic for GicV3 { 
    /// Initialises the GIC.
    fn init_primary(&mut self) {
        self.gicd.init();
        self.per_cpu_init();
    }

    fn per_cpu_init(&mut self) {
        self.gicr.init();
        self.cpu_sys_reg_init();
    }

    /// Enables the interrupt with the given ID.
    fn enable_interrupt(&mut self, intid: IntId) {
        let index = intid.0 / 32;
        let bit = 1 << (intid.0 % 32);

        if intid.is_private() {
            self.gicr.sgi_regs().ISENABLER[index].set(bit);
        } else {
            self.gicd.regs().ISENABLER[index].set(bit);
        }
    }

    fn disable_interrupt(&mut self, intid: IntId) {
        let index = intid.0 / 32;
        let bit = 1 << (intid.0 % 32);

        if intid.is_private() {
            self.gicr.sgi_regs().ICENABLER[index].set(bit);
        } else {
            self.gicd.regs().ICENABLER[index].set(bit);
        }
    }

    /// Configures the trigger type for the interrupt with the given ID.
    fn set_trigger(&mut self, intid: IntId, tm: TriggerMode) {
        if intid.is_private() {
            self.gicr.set_trigger(intid.0, tm);
        } else {
            self.gicd.set_trigger(intid.0, tm);
        }
    }

    /// Gets the ID of the highest priority signalled interrupt, and acknowledges it.
    ///
    /// Returns `None` if there is no pending interrupt of sufficient priority.
    fn get_and_acknowledge_interrupt(&self) -> Option<IntId> {
        // SAFETY: Reading this system register doesn't access memory in any way.
        let intid = unsafe { read_sysreg!(icc_iar1_el1) } as usize;
        if intid == IntId::SPECIAL_START {
            None
        } else {
            Some(IntId(intid))
        }
    }

    /// Informs the interrupt controller that the CPU has completed processing the given interrupt.
    /// This drops the interrupt priority and deactivates the interrupt.
    fn end_interrupt(&self, intid: IntId) {
        // SAFETY: Writing to this system register doesn't access memory in any way.
        unsafe { write_sysreg!(icc_eoir1_el1, intid.0 as u64) }
    }
}

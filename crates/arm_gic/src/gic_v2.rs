//! Types and definitions for GICv2.

use core::ptr::NonNull;

use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

pub const PPI_BASE: usize = 16;
pub const SPI_BASE: usize = 32;

const MAX_IRQ_DEFAULT: usize = 1024;

register_structs! {
    #[allow(non_snake_case)]
    GicDistributorRegs {
        /// Distributor Control Register.
        (0x0000 => CTLR: ReadWrite<u32>),
        /// Interrupt Controller Type Register.
        (0x0004 => TYPER: ReadOnly<u32>),
        /// Distributor Implementer Identification Register.
        (0x0008 => IIDR: ReadOnly<u32>),
        (0x000c => _reserved_0),
        /// Interrupt Group Registers.
        (0x0080 => IGROUPR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Enable Registers.
        (0x0100 => ISENABLER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Enable Registers.
        (0x0180 => ICENABLER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Pending Registers.
        (0x0200 => ISPENDR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Pending Registers.
        (0x0280 => ICPENDR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Active Registers.
        (0x0300 => ISACTIVER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Active Registers.
        (0x0380 => ICACTIVER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Priority Registers.
        (0x0400 => IPRIORITYR: [ReadWrite<u32>; 0x100]),
        /// Interrupt Processor Targets Registers.
        (0x0800 => ITARGETSR: [ReadWrite<u32>; 0x100]),
        /// Interrupt Configuration Registers.
        (0x0c00 => ICFGR: [ReadWrite<u32>; 0x40]),
        (0x0d00 => _reserved_1),
        /// Software Generated Interrupt Register.
        (0x0f00 => SGIR: WriteOnly<u32>),
        (0x0f04 => @END),
    }
}

register_structs! {
    #[allow(non_snake_case)]
    GicCpuInterfaceRegs {
        /// CPU Interface Control Register.
        (0x0000 => CTLR: ReadWrite<u32>),
        /// Interrupt Priority Mask Register.
        (0x0004 => PMR: ReadWrite<u32>),
        /// Binary Point Register.
        (0x0008 => BPR: ReadWrite<u32>),
        /// Interrupt Acknowledge Register.
        (0x000c => IAR: ReadOnly<u32>),
        /// End of Interrupt Register.
        (0x0010 => EOIR: WriteOnly<u32>),
        /// Running Priority Register.
        (0x0014 => RPR: ReadOnly<u32>),
        /// Highest Priority Pending Interrupt Register.
        (0x0018 => HPPIR: ReadOnly<u32>),
        (0x001c => _reserved_1),
        /// CPU Interface Identification Register.
        (0x00fc => IIDR: ReadOnly<u32>),
        (0x0100 => _reserved_2),
        /// Deactivate Interrupt Register.
        (0x1000 => DIR: WriteOnly<u32>),
        (0x1004 => @END),
    }
}

pub enum TriggerMode {
    Edge = 0,
    Level = 1,
}

pub enum Polarity {
    ActiveHigh = 0,
    ActiveLow = 1,
}

pub struct GicDistributor {
    base: NonNull<GicDistributorRegs>,
    max_irqs: usize,
}

pub struct GicCpuInterface {
    base: NonNull<GicCpuInterfaceRegs>,
}

unsafe impl Send for GicDistributor {}
unsafe impl Sync for GicDistributor {}

unsafe impl Send for GicCpuInterface {}
unsafe impl Sync for GicCpuInterface {}

impl GicDistributor {
    pub const fn new(base: *mut u8) -> Self {
        Self {
            base: NonNull::new(base).unwrap().cast(),
            max_irqs: MAX_IRQ_DEFAULT,
        }
    }

    const fn regs(&self) -> &GicDistributorRegs {
        unsafe { self.base.as_ref() }
    }

    pub fn cpu_num(&self) -> usize {
        ((self.regs().TYPER.get() as usize >> 5) & 0b111) + 1
    }

    pub fn max_irqs(&self) -> usize {
        ((self.regs().TYPER.get() as usize & 0b11111) + 1) * 32
    }

    pub fn configure_interrupt(&mut self, vector: usize, tm: TriggerMode, pol: Polarity) {
        // Only configurable for SPI interrupts
        if vector >= self.max_irqs || vector < SPI_BASE {
            return;
        }
        // TODO: polarity should actually be configure through a GPIO controller
        if !matches!(pol, Polarity::ActiveHigh) {
            return;
        }

        // type is encoded with two bits, MSB of the two determine type
        // 16 irqs encoded per ICFGR register
        let reg_idx = vector >> 4;
        let bit_shift = ((vector & 0xf) << 1) + 1;
        let mut reg_val = self.regs().ICFGR[reg_idx].get();
        match tm {
            TriggerMode::Edge => reg_val |= 1 << bit_shift,
            TriggerMode::Level => reg_val &= !(1 << bit_shift),
        }
        self.regs().ICFGR[reg_idx].set(reg_val);
    }

    pub fn set_enable(&mut self, vector: usize, enable: bool) {
        if vector >= self.max_irqs {
            return;
        }
        let reg = vector / 32;
        let mask = 1 << (vector % 32);
        if enable {
            self.regs().ISENABLER[reg].set(mask);
        } else {
            self.regs().ICENABLER[reg].set(mask);
        }
    }

    pub fn init(&mut self) {
        let max_irqs = self.max_irqs();
        assert!(max_irqs <= MAX_IRQ_DEFAULT);
        self.max_irqs = max_irqs;

        // Disable all interrputs
        for i in (0..max_irqs).step_by(32) {
            self.regs().ICENABLER[i / 32].set(u32::MAX);
            self.regs().ICPENDR[i / 32].set(u32::MAX);
        }
        if self.cpu_num() > 1 {
            for i in (SPI_BASE..max_irqs).step_by(4) {
                // Set external interrupts to target cpu 0
                self.regs().ITARGETSR[i / 4].set(0x01_01_01_01);
            }
        }
        // Initialize all the SPIs to edge triggered
        for i in SPI_BASE..max_irqs {
            self.configure_interrupt(i, TriggerMode::Edge, Polarity::ActiveHigh);
        }

        // enable GIC0
        self.regs().CTLR.set(1);
    }
}

impl GicCpuInterface {
    pub const fn new(base: *mut u8) -> Self {
        Self {
            base: NonNull::new(base).unwrap().cast(),
        }
    }

    const fn regs(&self) -> &GicCpuInterfaceRegs {
        unsafe { self.base.as_ref() }
    }

    pub fn iar(&self) -> u32 {
        self.regs().IAR.get()
    }

    pub fn eoi(&self, iar: u32) {
        self.regs().EOIR.set(iar);
    }

    pub fn handle_irq<F>(&self, handler: F)
    where
        F: FnOnce(u32),
    {
        let iar = self.iar();
        let vector = iar & 0x3ff;
        if vector < 1020 {
            handler(vector);
            self.eoi(iar);
        } else {
            // spurious
        }
    }

    pub fn init(&self) {
        // enable GIC0
        self.regs().CTLR.set(1);
        // unmask interrupts at all priority levels
        self.regs().PMR.set(0xff);
    }
}

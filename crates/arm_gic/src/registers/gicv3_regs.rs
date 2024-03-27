//! Types and definitions for GICv3.
//!
//! The official documentation: <https://developer.arm.com/documentation/ihi0069/latest//>

use bitflags::bitflags;
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

bitflags! {
    #[repr(transparent)]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct GicdCtlr: u32 {
        const RWP = 1 << 31;
        const nASSGIreq = 1 << 8;
        const E1NWF = 1 << 7;
        const DS = 1 << 6;
        const ARE_NS = 1 << 5;
        const ARE_S = 1 << 4;
        const EnableGrp1S = 1 << 2;
        const EnableGrp1NS = 1 << 1;
        const EnableGrp0 = 1 << 0;
    }
}

register_structs! {
    /// GIC Distributor registers.
    #[allow(non_snake_case)]
    pub(crate) GicDistributorRegs {
        /// Distributor Control Register.
        (0x0000 => pub(crate) CTLR: ReadWrite<u32>),
        /// Interrupt Controller Type Register.
        (0x0004 => pub(crate) TYPER: ReadOnly<u32>),
        /// Distributor Implementer Identification Register.
        (0x0008 => pub(crate) IIDR: ReadOnly<u32>),
        /// Interrupt controller type register 2.
        (0x000c => pub(crate) TYPER2: ReadOnly<u32>),
        /// Error reporting status register.
        (0x0010 => pub(crate) STATUSR: ReadOnly<u32>),
        (0x0014 => _reserved0),
        /// Implementation defined registers.
        (0x0020 => implementation_defined1: [ReadWrite<u32>; 0x08]),
        /// Set SPI register.
        (0x0040 => pub(crate) SETSPI_NSR: ReadWrite<u32>),
        (0x0044 => _reserved1),
        /// Clear SPI register.
        (0x0048 => pub(crate) CLRSPI_NSR: ReadWrite<u32>),
        (0x004c => _reserved2),
        /// Set SPI secure register.
        (0x0050 => pub(crate) SETSPI_SR: ReadWrite<u32>),
        (0x0054 => _reserved3),
        /// clear SPI secure register.
        (0x0058 => pub(crate) CLRSPI_SR: ReadWrite<u32>),
        (0x005c => _reserved4),
        /// Interrupt Group Registers.
        (0x0080 => pub(crate) IGROUPR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Enable Registers.
        (0x0100 => pub(crate) ISENABLER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Enable Registers.
        (0x0180 => pub(crate) ICENABLER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Pending Registers.
        (0x0200 => pub(crate) ISPENDR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Pending Registers.
        (0x0280 => pub(crate) ICPENDR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Active Registers.
        (0x0300 => pub(crate) ISACTIVER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Active Registers.
        (0x0380 => pub(crate) ICACTIVER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Priority Registers.
        (0x0400 => pub(crate) IPRIORITYR: [ReadWrite<u32>; 0x100]),
        /// Interrupt Processor Targets Registers.
        (0x0800 => pub(crate) ITARGETSR: [ReadWrite<u32>; 0x100]),
        /// Interrupt Configuration Registers.
        (0x0c00 => pub(crate) ICFGR: [ReadWrite<u32>; 0x40]),
        /// Interrupt group modifier registers.
        (0x0d00 => pub(crate) IGRPMODR: [ReadWrite<u32>; 0x20]),
        (0x0d80 => _reserved5),
        /// Non-secure access control registers.
        (0x0e00 => pub(crate) NSACR: [ReadWrite<u32>; 0x40]),
        /// Software generated interrupt register.
        (0x0f00 => pub(crate) SGIR: WriteOnly<u32>),
        (0x0f04 => _reserved6),
        /// SGI clear-pending registers.
        (0x0f10 => pub(crate) CPENDSGIR: [ReadWrite<u32>; 0x04]),
        /// SGI set-pending registers.
        (0x0f20 => pub(crate) SPENDSGIR: [ReadWrite<u32>; 0x04]),
        (0x0f30 => _reserved7),
        /// Non-maskable interrupt registers.
        (0x0f80 => pub(crate) INMIR: [ReadWrite<u32>; 0x20]),
        /// Interrupt group registers for extended SPI range.
        (0x1000 => pub(crate) IGROUPRnE: [ReadWrite<u32>; 0x20]),
        (0x1080 => _reserved8),
        /// Interrupt set-enable registers for extended SPI range.
        (0x1200 => pub(crate)  ISENABLERnE: [ReadWrite<u32>; 0x20]),
        (0x1280 => _reserved9),
        /// Interrupt clear-enable registers for extended SPI range.
        (0x1400 => pub(crate) ICENABLERnE: [ReadWrite<u32>; 0x20]),
        (0x1480 => _reserved10),
        /// Interrupt set-pending registers for extended SPI range.
        (0x1600 => pub(crate) ISPENDRnE: [ReadWrite<u32>; 0x20]),
        (0x1680 => _reserved11),
        /// Interrupt clear-pending registers for extended SPI range.
        (0x1800 => pub(crate) ICPENDRnE: [ReadWrite<u32>; 0x20]),
        (0x1880 => _reserved12),
        /// Interrupt set-active registers for extended SPI range.
        (0x1a00 => pub(crate) ISACTIVERnE: [ReadWrite<u32>; 0x20]),
        (0x1a80 => _reserved13),
        /// Interrupt clear-active registers for extended SPI range.
        (0x1c00 => pub(crate) ICACTIVERnE: [ReadWrite<u32>; 0x20]),
        (0x1c80 => _reserved14),
        /// Interrupt priority registers for extended SPI range.
        (0x2000 => pub(crate) IPRIORITYRnE: [ReadWrite<u32>; 0x100]),
        (0x2400 => _reserved15),
        /// Extended SPI configuration registers.
        (0x3000 => pub(crate) ICFGRnE: [ReadWrite<u32>; 0x40]),
        (0x3100 => _reserved16),
        /// Interrupt group modifier registers for extended SPI range.
        (0x3400 => pub(crate) IGRPMODRnE: [ReadWrite<u32>; 0x20]),
        (0x3480 => _reserved17),
        /// Non-secure access control registers for extended SPI range.
        (0x3600 => pub(crate) NSACRnE: [ReadWrite<u32>; 0x20]),
        (0x3680 => _reserved18),
        /// Non-maskable interrupt registers for extended SPI range.
        (0x3b00 => pub(crate) INMRnE: [ReadWrite<u32>; 0x20]),
        (0x3b80 => _reserved19),
        /// Interrupt routing registers.
        (0x6000 => pub(crate) IROUTER: [ReadWrite<u64>; 1024]),
        /// Interrupt routing registers for extended SPI range.
        (0x8000 => pub(crate) IROUTERnE: [ReadWrite<u64>; 1024]),
        (0xa000 => _reserved21),
        /// Implementation defined registers.
        (0xc000 => implementation_defined2: [ReadWrite<u32>; 0xff4]),
        /// ID registers.
        (0xffd0 => implementation_defined3: [ReadOnly<u32>; 6]),
        (0xffe8 => pub(crate)  PIDR2:ReadOnly<u32>),
        (0xffec => implementation_defined4: [ReadOnly<u32>; 5]),
        (0x10000 => @END),
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct WakerFlags: u32 {
        const CHILDREN_ASLEEP = 1 << 2;
        const PROCESSOR_SLEEP = 1 << 1;
    }
}

register_structs! {
    /// GIC Redistributor registers.
    #[allow(non_snake_case)]
    pub(crate) GicRedistributorRegs {
        /// Redistributor control register.
        (0x0000 => pub(crate) CTLR: ReadWrite<u32>),
        /// Implementer identification register.
        (0x0004 => pub(crate) IIDR: ReadOnly<u32>),
        /// Redistributor type register.
        (0x0008 => pub(crate) TYPER: ReadOnly<u64>),
        /// Error reporting status register.
        (0x0010 => pub(crate) STATUSR: ReadWrite<u32>),
        /// Redistributor wake register.
        (0x0014 => pub(crate) WAKER: ReadWrite<u32>),
        /// Report maximum PARTID and PMG register.
        (0x0018 => pub(crate) MPAMIDR: ReadOnly<u32>),
        /// Set PARTID and PMG register.
        (0x001c => pub(crate) PARTIDR: ReadWrite<u32>),
        /// Implementation defined registers.
        (0x0020 => implementation_defined1: [ReadOnly<u32>; 0x08]),
        /// Set LPI pending register.
        (0x0040 => pub(crate) SETLPIR: WriteOnly<u64>),
        /// Clear LPI pending register.
        (0x0048 => pub(crate) CLRLPIR: WriteOnly<u64>),
        (0x0050 => _reserved0: [ReadOnly<u32>; 8]),
        /// Redistributor properties base address register.
        (0x0070 => pub(crate) PROPBASER: ReadWrite<u64>),
        /// Redistributor LPI pending table base address register.
        (0x0078 => pub(crate) PENDBASER: ReadWrite<u64>),
        (0x0080 => _reserved1: [ReadOnly<u32>; 8]),
        /// Redistributor invalidate LPI register.
        (0x00a0 => pub(crate) INVLPIR: ReadWrite<u64>),
        (0x00a8 => _reserved2: ReadOnly<u64>),
        /// Redistributor invalidate all register.
        (0x00b0 => pub(crate) INVALLR: ReadWrite<u64>),
        (0x00b8 => _reserved3: ReadOnly<u64>),
        /// Redistributor synchronize register.
        (0x00c0 => pub(crate) SYNCR: ReadOnly<u32>),
        (0x00c4 => _reserved4: [ReadOnly<u32>; 0x0f]),
        /// Implementation defined registers.
        (0x0100 => pub(crate) implementation_defined2: WriteOnly<u64>),
        (0x0108 => _reserved5: ReadOnly<u64>),
        /// Implementation defined registers.
        (0x0110 => implementation_defined3: WriteOnly<u64>),
        (0x0118 => _reserved6: [ReadOnly<u32>; 0x2fba]),
        /// Implementation defined registers.
        (0xc000 => implementation_defined4: [WriteOnly<u32>; 0xff4]),
        /// ID registers.
        (0xffd0 => pub(crate) IDREGS: [ReadOnly<u32>; 0x0c]),
        (0x10000 => @END),
    }
}

register_structs! {
    /// GIC Distributor registers.
    #[allow(non_snake_case)]
    pub(crate) GicSgiRegs {
        (0x0000 => _reserved0: [ReadOnly<u32>; 32]),
        /// Interrupt group register 0
        /// Interrupt group registers for extended PPI range.
        (0x0080 => pub(crate) IGROUPR0: [ReadWrite<u32>; 3]),
        (0x008c => _reserved1: [ReadOnly<u32>; 29]),
        /// Interrupt set-enable register 0.
        /// Interrupt set-enable registers for extended PPI range.
        (0x0100 => pub(crate) ISENABLER: [ReadWrite<u32>; 1+2]),
        (0x010c => _reserved2: [ReadOnly<u32>; 29]),
        /// Interrupt clear-enable register 0.
        /// Interrupt clear-enable registers for extended PPI range.
        (0x0180 => pub(crate) ICENABLER: [ReadWrite<u32>;1+2]),
        (0x018c => _reserved3: [ReadOnly<u32>; 29]),
        /// Interrupt set-pending register 0.
        /// Interrupt set-pending registers for extended PPI range.
        (0x0200 => pub(crate) ISPENDR: [ReadWrite<u32>; 1+2]),
        (0x020c => _reserved4: [ReadOnly<u32>; 29]),
        /// Interrupt clear-pending register 0.
        /// Interrupt clear-pending registers for extended PPI range.
        (0x0280 => pub(crate) ICPENDR: [ReadWrite<u32>; 1+2] ),
        (0x028c => _reserved5: [ReadOnly<u32>; 29]),
        /// Interrupt set-active register 0.
        /// Interrupt set-active registers for extended PPI range.
        (0x0300 => pub(crate) ISACTIVER: [ReadWrite<u32>;1+2]),
        (0x030c => _reserved6: [ReadOnly<u32>; 29]),
        /// Interrupt clear-active register 0.
        /// Interrupt clear-active registers for extended PPI range.
        (0x0380 => pub(crate) ICACTIVER: [ReadWrite<u32>;1+2]),
        (0x038c => _reserved7: [ReadOnly<u32>; 29]),
        /// Interrupt priority registers Interrupt priority registers for extended PPI range.
        (0x0400 => pub(crate) IPRIORITYR: [ReadWrite<u32>; 8+16]),
        (0x0460 => _reserved8: [ReadOnly<u32>; 488]),
        /// SGI configuration register,
        /// PPI configuration register and extended PPI configuration registers.
        (0x0c00 => pub(crate) ICFGR: [ReadWrite<u32>; 6]),
        (0x0c18 => _reserved9: [ReadOnly<u32>; 58]),
        /// Interrupt group modifier register 0.
        /// Interrupt group modifier registers for extended PPI range.
        (0x0d00 => pub(crate) IGRPMODR: [ReadWrite<u32>; 1+2]),
        (0x0d0c => _reserved10: [ReadOnly<u32>; 61]),
        /// Non-secure access control register.
        (0x0e00 => pub(crate) NSACR: ReadWrite<u32>),
        (0x0e04 => _reserved11: [ReadOnly<u32>; 95]),
        /// Non-maskable interrupt register for PPIs.
        (0x0f80 => pub(crate) INMIR0: ReadWrite<u32>),
        /// Non-maskable interrupt register for extended PPIs.
        (0x0f84 => pub(crate) INMIRnE: [ReadWrite<u32>; 31]),
        (0x1000 => _reserved12: [ReadOnly<u32>; 11264]),
        /// Implementation defined registers.
        (0xc000 => implementation_defined: [ReadOnly<u32>; 0xff4]),
        (0xffd0 => _reserved13: [ReadOnly<u32>; 0x0c]),
        (0x10000 => @END),
    }
}

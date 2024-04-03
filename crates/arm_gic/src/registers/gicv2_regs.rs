//! Types and definitions for GICv2.
//!
//! The official documentation: <https://developer.arm.com/documentation/ihi0048/latest/>

use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

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
        (0x000c => _reserved_0),
        /// Interrupt Group Registers.
        (0x0080 => pub(crate) IGROUPRn: [ReadWrite<u32>; 0x20]),
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
        (0x0d00 => _reserved_1),
        /// Non-secure Access Control Registers, optional
        (0x0e00 => pub(crate) NSACR: [ReadWrite<u32>; 0x40]),
        /// Software Generated Interrupt Register.
        (0x0f00 => pub(crate) SGIR: WriteOnly<u32>),
        (0x0f04 => _reserved_2),
        /// SGI Clear-Pending Registers
        (0x0f10 => pub(crate) CPENDSGIR: [ReadWrite<u32>; 4]),
        /// SGI Set-Pending Registers
        (0x0f20 => pub(crate) SPENDSGIR: [ReadWrite<u32>; 4]),
        (0x0f30 => @END),
    }
}

register_structs! {
    /// GIC CPU Interface registers.
    #[allow(non_snake_case)]
    pub(crate) GicCpuInterfaceRegs {
        /// CPU Interface Control Register.
        (0x0000 => pub(crate) CTLR: ReadWrite<u32>),
        /// Interrupt Priority Mask Register.
        (0x0004 => pub(crate) PMR: ReadWrite<u32>),
        /// Binary Point Register.
        (0x0008 => pub(crate) BPR: ReadWrite<u32>),
        /// Interrupt Acknowledge Register.
        (0x000c => pub(crate) IAR: ReadOnly<u32>),
        /// End of Interrupt Register.
        (0x0010 => pub(crate) EOIR: WriteOnly<u32>),
        /// Running Priority Register.
        (0x0014 => pub(crate) RPR: ReadOnly<u32>),
        /// Highest Priority Pending Interrupt Register.
        (0x0018 => pub(crate) HPPIR: ReadOnly<u32>),
        /// Aliased Binary Point Register
        (0x001c => pub(crate) ABPR: ReadWrite<u32>),
        /// Aliased Interrupt Acknowledge Register
        (0x0020 => pub(crate) AIAR: ReadWrite<u32>),
        /// Aliased End of Interrupt Register
        (0x0024 => pub(crate) AEOIR: WriteOnly<u32>),
        /// Aliased Highest Priority Pending Interrupt Register
        (0x0028 => pub(crate) AHPPIR: WriteOnly<u32>),
        (0x002c => _reserved_1),
        /// Active Priorities Registers
        (0x00d0 => pub(crate) APRn: [ReadWrite<u32>; 4]),
        /// Non-secure Active Priorities Registers
        (0x00E0 => pub(crate) NSAPRn: [ReadWrite<u32>; 4]),
        (0x00f0 => _reserved_2),
        /// CPU Interface Identification Register.
        (0x00fc => pub(crate) IIDR: ReadOnly<u32>),
        (0x0100 => _reserved_3),
        /// Deactivate Interrupt Register.
        (0x1000 => pub(crate) DIR: WriteOnly<u32>),
        (0x1004 => @END),
    }
}

// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Copyright (c) 2018-2023 by the author(s)
//
// Author(s):
//   - Andre Richter <andre.o.richter@gmail.com>
//   - Berkus Decker <berkus+github@metta.systems>

//! Exception Syndrome Register - EL1
//!
//! Holds syndrome information for an exception taken to EL1.

use tock_registers::{interfaces::Readable, register_bitfields};

register_bitfields! {u64,
    pub ESR_EL1 [
        /// Exception Class. Indicates the reason for the exception that this register holds
        /// information about.
        ///
        /// For each EC value, the table references a subsection that gives information about:
        ///   - The cause of the exception, for example the configuration required to enable the
        ///     trap.
        ///   - The encoding of the associated ISS.
        ///
        /// Incomplete listing - to be done.
        EC  OFFSET(26) NUMBITS(6) [
            Unknown               = 0b00_0000,
            TrappedWFIorWFE       = 0b00_0001,
            TrappedMCRorMRC       = 0b00_0011, // A32
            TrappedMCRRorMRRC     = 0b00_0100, // A32
            TrappedMCRorMRC2      = 0b00_0101, // A32
            TrappedLDCorSTC       = 0b00_0110, // A32
            TrappedFP             = 0b00_0111,
            TrappedMRRC           = 0b00_1100, // A32
            BranchTarget          = 0b00_1101,
            IllegalExecutionState = 0b00_1110,
            SVC32                 = 0b01_0001, // A32
            SVC64                 = 0b01_0101,
            HVC64                 = 0b01_0110,
            SMC64                 = 0b01_0111,
            TrappedMsrMrs         = 0b01_1000,
            TrappedSve            = 0b01_1001,
            PointerAuth           = 0b01_1100,
            InstrAbortLowerEL     = 0b10_0000,
            InstrAbortCurrentEL   = 0b10_0001,
            PCAlignmentFault      = 0b10_0010,
            DataAbortLowerEL      = 0b10_0100,
            DataAbortCurrentEL    = 0b10_0101,
            SPAlignmentFault      = 0b10_0110,
            TrappedFP32           = 0b10_1000, // A32
            TrappedFP64           = 0b10_1100,
            SError                = 0b10_1111,
            BreakpointLowerEL     = 0b11_0000,
            BreakpointCurrentEL   = 0b11_0001,
            SoftwareStepLowerEL   = 0b11_0010,
            SoftwareStepCurrentEL = 0b11_0011,
            WatchpointLowerEL     = 0b11_0100,
            WatchpointCurrentEL   = 0b11_0101,
            Bkpt32                = 0b11_1000, // A32 BKTP instruction
            Brk64                 = 0b11_1100  // A64 BRK instruction
        ],

        /// Instruction Length for synchronous exceptions.
        IL  OFFSET(25) NUMBITS(1) [],

        /// Instruction Specific Syndrome. Architecturally, this field can be defined independently
        /// for each defined Exception class. However, in practice, some ISS encodings are used for
        /// more than one Exception class.
        ISS OFFSET(0)  NUMBITS(25) []
    ]
}

pub struct Reg;

impl Readable for Reg {
    type T = u64;
    type R = ESR_EL1::Register;

    sys_coproc_read_raw!(u64, "ESR_EL1", "x");
}

pub const ESR_EL1: Reg = Reg {};

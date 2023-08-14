// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Copyright (c) 2018-2023 by the author(s)
//
// Author(s):
//   - Andre Richter <andre.o.richter@gmail.com>

//! Multiprocessor Affinity Register - EL1
//!
//! In a multiprocessor system, provides an additional PE identification mechanism for scheduling
//! purposes.

use tock_registers::interfaces::Readable;

pub struct Reg;

impl Readable for Reg {
    type T = u64;
    type R = ();

    sys_coproc_read_raw!(u64, "MPIDR_EL1", "x");
}

pub const MPIDR_EL1: Reg = Reg {};

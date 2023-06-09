//! Simple RISC-V SBI runtime primitives
#![no_std]
#[cfg_attr(not(feature = "legacy"), deny(missing_docs))]
// §3
mod binary;
// §4
mod base;
// §5
#[cfg(feature = "legacy")]
pub mod legacy;
// §6
mod time;
// §7
mod spi;
// §8
mod rfnc;
// §9
mod hsm;
// §10
mod srst;
// §11
mod pmu;

pub use base::*;
pub use binary::*;
pub use hsm::*;
pub use pmu::*;
pub use rfnc::*;
pub use spi::*;
pub use srst::*;
pub use time::*;

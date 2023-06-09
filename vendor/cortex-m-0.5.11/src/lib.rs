//! Low level access to Cortex-M processors
//!
//! This crate provides:
//!
//! - Access to core peripherals like NVIC, SCB and SysTick.
//! - Access to core registers like CONTROL, MSP and PSR.
//! - Interrupt manipulation mechanisms
//! - Safe wrappers around Cortex-M specific instructions like `bkpt`
//!
//! # Optional features
//!
//! ## `inline-asm`
//!
//! When this feature is enabled the implementation of all the functions inside the `asm` and
//! `register` modules use inline assembly (`asm!`) instead of external assembly (FFI into separate
//! assembly files pre-compiled using `arm-none-eabi-gcc`). The advantages of enabling `inline-asm`
//! are:
//!
//! - Reduced overhead. FFI eliminates the possibility of inlining so all operations include a
//! function call overhead when `inline-asm` is not enabled.
//!
//! - Some of the `register` API only becomes available only when `inline-asm` is enabled. Check the
//! API docs for details.
//!
//! The disadvantage is that `inline-asm` requires a nightly toolchain.
//!
//! ## `const-fn`
//!
//! Enabling this feature turns the `Mutex.new` constructor into a `const fn`.
//!
//! This feature requires a nightly toolchain.

#![cfg_attr(feature = "inline-asm", feature(asm))]
#![deny(missing_docs)]
#![no_std]

extern crate aligned;
extern crate bare_metal;
extern crate volatile_register;
extern crate cortex_m_0_6;

#[macro_use]
mod macros;

pub use cortex_m_0_6::{asm, interrupt, register};
#[cfg(not(armv6m))]
pub use cortex_m_0_6::itm;

pub mod peripheral;

pub use peripheral::Peripherals;

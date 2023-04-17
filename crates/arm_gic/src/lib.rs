//! ARM Generic Interrupt Controller (GIC) driver.

#![no_std]
#![feature(const_ptr_as_ref)]
#![feature(const_option)]
#![feature(const_nonnull_new)]

pub mod gic_v2;

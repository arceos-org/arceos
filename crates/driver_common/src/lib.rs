//! Device driver interfaces used by ArceOS. It provides common traits and types
//! for implementing a device driver.
//!
//! You have to use this crate with the following crates for corresponding
//! device types:
//!
//! - [`driver_block`]: Common traits for block storage drivers.
//! - [`driver_display`]: Common traits and types for graphics display drivers.
//! - [`driver_net`]: Common traits and types for network (NIC) drivers.
//!
//! [`driver_block`]: ../driver_block/index.html
//! [`driver_display`]: ../driver_display/index.html
//! [`driver_net`]: ../driver_net/index.html

#![no_std]
#![feature(const_trait_impl)]

/// All supported device types.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DeviceType {
    /// Block device.
    Block,
    /// Character device.
    Char,
    /// Network device.
    Net,
    /// Display device.
    Display,
}

/// The error type for device operation failures.
#[derive(Debug)]
pub enum DevError {
    /// An entity already exists.
    AlreadyExists,
    /// Try again, for non-blocking APIs.
    Again,
    /// Bad internal state.
    BadState,
    /// Invalid parameter/argument.
    InvalidParam,
    /// Input/output error.
    Io,
    /// Not enough space/cannot allocate memory (DMA).
    NoMemory,
    /// Device or resource is busy.
    ResourceBusy,
    /// This operation is unsupported or unimplemented.
    Unsupported,
}

/// A specialized `Result` type for device operations.
pub type DevResult<T = ()> = Result<T, DevError>;

/// Common operations that require all device drivers to implement.
#[const_trait]
pub trait BaseDriverOps: Send + Sync {
    /// The name of the device.
    fn device_name(&self) -> &str;

    /// The type of the device.
    fn device_type(&self) -> DeviceType;
}

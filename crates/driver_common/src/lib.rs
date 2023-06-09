//! Device driver interfaces used by [ArceOS][1]. It provides common traits and
//! types for implementing a device driver.
//!
//! You have to use this crate with the following crates for corresponding
//! device types:
//!
//! - [`driver_block`][2]: Common traits for block storage drivers.
//! - [`driver_display`][3]: Common traits and types for graphics display drivers.
//! - [`driver_net`][4]: Common traits and types for network (NIC) drivers.
//!
//! [1]: https://github.com/rcore-os/arceos
//! [2]: ../driver_block/index.html
//! [3]: ../driver_display/index.html
//! [4]: ../driver_net/index.html

#![no_std]
#![feature(const_trait_impl)]

/// All supported device types.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DeviceType {
    /// Block storage device (e.g., disk).
    Block,
    /// Character device (e.g., serial port).
    Char,
    /// Network device (e.g., ethernet card).
    Net,
    /// Graphic display device (e.g., GPU)
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

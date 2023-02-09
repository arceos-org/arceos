#![no_std]
#![feature(const_trait_impl)]

#[derive(Debug)]
pub enum DeviceType {
    Block,
    Char,
    Net,
    Display,
}

#[derive(Debug)]
pub enum DevError {
    /// An entity already exists.
    AlreadyExists,
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

pub type DevResult<T = ()> = Result<T, DevError>;

#[const_trait]
pub trait BaseDriverOps: Send + Sync {
    fn device_name(&self) -> &str;
    fn device_type(&self) -> DeviceType;
}

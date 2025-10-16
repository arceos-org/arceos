#[cfg_attr(feature = "dyn", path = "dyn.rs")]
#[cfg_attr(not(feature = "dyn"), path = "static.rs")]
mod imp;

use alloc::vec::Vec;

use axdriver_base::{BaseDriverOps, DeviceType};
pub use imp::*;

/// A unified enum that represents different categories of devices.
#[allow(clippy::large_enum_variant)]
pub enum AxDeviceEnum {
    /// Network card device.
    #[cfg(feature = "net")]
    Net(AxNetDevice),
    /// Block storage device.
    #[cfg(feature = "block")]
    Block(AxBlockDevice),
    /// Graphic display device.
    #[cfg(feature = "display")]
    Display(AxDisplayDevice),
    /// Graphic input device.
    #[cfg(feature = "input")]
    Input(AxInputDevice),
    #[cfg(feature = "vsock")]
    Vsock(AxVsockDevice),
}

impl BaseDriverOps for AxDeviceEnum {
    #[inline]
    #[allow(unreachable_patterns)]
    fn device_type(&self) -> DeviceType {
        match self {
            #[cfg(feature = "net")]
            Self::Net(_) => DeviceType::Net,
            #[cfg(feature = "block")]
            Self::Block(_) => DeviceType::Block,
            #[cfg(feature = "display")]
            Self::Display(_) => DeviceType::Display,
            #[cfg(feature = "input")]
            Self::Input(_) => DeviceType::Input,
            #[cfg(feature = "vsock")]
            Self::Vsock(_) => DeviceType::Vsock,
            _ => unreachable!(),
        }
    }

    #[inline]
    #[allow(unreachable_patterns)]
    fn device_name(&self) -> &str {
        match self {
            #[cfg(feature = "net")]
            Self::Net(dev) => dev.device_name(),
            #[cfg(feature = "block")]
            Self::Block(dev) => dev.device_name(),
            #[cfg(feature = "display")]
            Self::Display(dev) => dev.device_name(),
            #[cfg(feature = "input")]
            Self::Input(dev) => dev.device_name(),
            #[cfg(feature = "vsock")]
            Self::Vsock(dev) => dev.device_name(),
            _ => unreachable!(),
        }
    }
}

/// A structure that contains all device drivers of a certain category.
pub struct AxDeviceContainer<D>(Vec<D>);

impl<D> AxDeviceContainer<D> {
    /// Returns number of devices in this container.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the container is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Takes one device out of the container (will remove it from the
    /// container).
    pub fn take_one(&mut self) -> Option<D> {
        self.0.pop()
    }

    /// Adds one device into the container.
    #[allow(dead_code)]
    pub(crate) fn push(&mut self, dev: D) {
        self.0.push(dev);
    }
}

impl<D> core::ops::Deref for AxDeviceContainer<D> {
    type Target = Vec<D>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<D> Default for AxDeviceContainer<D> {
    fn default() -> Self {
        Self(Default::default())
    }
}

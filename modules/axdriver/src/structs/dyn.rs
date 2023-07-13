#![allow(unused_imports)]

use crate::prelude::*;
use alloc::{boxed::Box, vec, vec::Vec};

/// The unified type of the NIC devices.
#[cfg(feature = "net")]
pub type AxNetDevice = Box<dyn NetDriverOps>;
/// The unified type of the block storage devices.
#[cfg(feature = "block")]
pub type AxBlockDevice = Box<dyn BlockDriverOps>;
/// The unified type of the graphics display devices.
#[cfg(feature = "display")]
pub type AxDisplayDevice = Box<dyn DisplayDriverOps>;

impl super::AxDeviceEnum {
    /// Constructs a network device.
    #[cfg(feature = "net")]
    pub fn from_net(dev: impl NetDriverOps + 'static) -> Self {
        Self::Net(Box::new(dev))
    }

    /// Constructs a block device.
    #[cfg(feature = "block")]
    pub fn from_block(dev: impl BlockDriverOps + 'static) -> Self {
        Self::Block(Box::new(dev))
    }

    /// Constructs a display device.
    #[cfg(feature = "display")]
    pub fn from_display(dev: impl DisplayDriverOps + 'static) -> Self {
        Self::Display(Box::new(dev))
    }
}

/// A structure that contains all device drivers of a certain category.
///
/// If the feature `dyn` is enabled, the inner type is [`Vec<D>`]. Otherwise,
/// the inner type is [`Option<D>`] and at most one device can be contained.
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

    /// Takes one device out of the container (will remove it from the container).
    pub fn take_one(&mut self) -> Option<D> {
        if self.is_empty() {
            None
        } else {
            Some(self.0.remove(0))
        }
    }

    /// Constructs the container from one device.
    pub fn from_one(dev: D) -> Self {
        Self(vec![dev])
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

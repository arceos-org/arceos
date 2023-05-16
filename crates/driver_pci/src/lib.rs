//! Structures and functions for PCI bus operations.
//!
//! Currently, it just re-exports structures from the crate [virtio-drivers][1]
//! and its module [`virtio_drivers::transport::pci::bus`][2].
//!
//! [1]: https://docs.rs/virtio-drivers/latest/virtio_drivers/
//! [2]: https://docs.rs/virtio-drivers/latest/virtio_drivers/transport/pci/bus/index.html

#![no_std]

pub use virtio_drivers::transport::pci::bus::{BarInfo, Cam, HeaderType, MemoryBarType, PciError};
pub use virtio_drivers::transport::pci::bus::{
    CapabilityInfo, Command, DeviceFunction, DeviceFunctionInfo, PciRoot, Status,
};

/// Used to allocate MMIO regions for PCI BARs.
pub struct PciRangeAllocator {
    _start: u64,
    end: u64,
    current: u64,
}

impl PciRangeAllocator {
    /// Creates a new allocator from a memory range.
    pub const fn new(base: u64, size: u64) -> Self {
        Self {
            _start: base,
            end: base + size,
            current: base,
        }
    }

    /// Allocates a memory region with the given size.
    ///
    /// The `size` should be a power of 2, and the returned value is also a
    /// multiple of `size`.
    pub fn alloc(&mut self, size: u64) -> Option<u64> {
        if !size.is_power_of_two() {
            return None;
        }
        let ret = align_up(self.current, size);
        if ret + size > self.end {
            return None;
        }

        self.current = ret + size;
        Some(ret)
    }
}

const fn align_up(addr: u64, align: u64) -> u64 {
    (addr + align - 1) & !(align - 1)
}

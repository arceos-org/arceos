//! [ArceOS](https://github.com/arceos-org/arceos) global DMA allocator.

#![no_std]

extern crate alloc;

mod dma;

use core::{alloc::Layout, ptr::NonNull};

use allocator::AllocResult;
use memory_addr::PhysAddr;

use self::dma::ALLOCATOR;

/// Converts a physical address to a bus address.
///
/// It assumes that there is a linear mapping with the offset
/// [`axconfig::plat::PHYS_BUS_OFFSET`], that maps all the physical memory
/// to the virtual space at the address plus the offset. So we have
/// `baddr = paddr + PHYS_BUS_OFFSET`.
#[inline]
pub const fn phys_to_bus(paddr: PhysAddr) -> BusAddr {
    BusAddr::new((paddr.as_usize() + axconfig::plat::PHYS_BUS_OFFSET) as u64)
}

/// Allocates **coherent** memory that meets Direct Memory Access (DMA)
/// requirements.
///
/// This function allocates a block of memory through the global allocator. The
/// memory pages must be contiguous, undivided, and have consistent read and
/// write access.
///
/// - `layout`: The memory layout, which describes the size and alignment
///   requirements of the requested memory.
///
/// Returns an [`DMAInfo`] structure containing details about the allocated
/// memory, such as the starting address and size. If it's not possible to
/// allocate memory meeting the criteria, returns [`None`].
///
/// # Safety
///
/// This function is unsafe because it directly interacts with the global
/// allocator, which can potentially cause memory leaks or other issues if not
/// used correctly.
pub unsafe fn alloc_coherent(layout: Layout) -> AllocResult<DMAInfo> {
    ALLOCATOR.lock().alloc_coherent(layout)
}

/// Frees coherent memory previously allocated.
///
/// This function releases the memory block that was previously allocated and
/// marked as coherent. It ensures proper deallocation and management of resources
/// associated with the memory block.
///
/// - `dma_info`: An instance of [`DMAInfo`] containing the details of the memory
///   block to be freed, such as its starting address and size.
///
/// # Safety
///
/// This function is unsafe because it directly interacts with the global allocator,
/// which can potentially cause memory leaks or other issues if not used correctly.
pub unsafe fn dealloc_coherent(dma: DMAInfo, layout: Layout) {
    ALLOCATOR.lock().dealloc_coherent(dma, layout)
}

/// A bus memory address.
///
/// It's a wrapper type around an [`u64`].
#[repr(transparent)]
#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct BusAddr(u64);

impl BusAddr {
    /// Converts an [`u64`] to a physical address.
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Converts the address to an [`u64`].
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for BusAddr {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl core::fmt::Debug for BusAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("BusAddr")
            .field(&format_args!("{:#X}", self.0))
            .finish()
    }
}

/// Represents information related to a DMA operation.
#[derive(Debug, Clone, Copy)]
pub struct DMAInfo {
    /// The address at which the CPU accesses this memory region. This address
    /// is a virtual memory address used by the CPU to access memory.
    pub cpu_addr: NonNull<u8>,
    /// Represents the physical address of this memory region on the bus. The DMA
    /// controller uses this address to directly access memory.
    pub bus_addr: BusAddr,
}

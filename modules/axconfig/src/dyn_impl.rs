//! Platform-specific constants and parameters for [ArceOS].
//!
//! Currently supported platform configs can be found in the [configs] directory of
//! the [ArceOS] root.
//!
//! [ArceOS]: https://github.com/arceos-org/arceos
//! [configs]: https://github.com/arceos-org/arceos/tree/main/configs

#![no_std]

/// Architecture identifier.
pub const ARCH: &str = "aarch64";
/// Platform package.
pub const PACKAGE: &str = "axplat-aarch64-generic";
/// Platform identifier.
pub const PLATFORM: &str = "aarch64-generic";
/// Stack size of each task.
pub const TASK_STACK_SIZE: usize = 0x40000;
/// Number of timer ticks per second (Hz). A timer tick may contain several timer
/// interrupts.
pub const TICKS_PER_SEC: usize = 100;
///
/// Device specifications
///
pub mod devices {
    /// MMIO regions with format (`base_paddr`, `size`).
    pub const MMIO_REGIONS: &[(usize, usize)] = &[
        (0xb000_0000, 0x1000_0000), // PCI config space
        (0xfe00_0000, 0xc0_0000),   // PCI devices
        (0xfec0_0000, 0x1000),      // IO APIC
        (0xfed0_0000, 0x1000),      // HPET
        (0xfee0_0000, 0x1000),      // Local APIC
    ];
    /// End PCI bus number.
    pub const PCI_BUS_END: usize = 0xff;
    /// Base physical address of the PCIe ECAM space.
    pub const PCI_ECAM_BASE: usize = 0xb000_0000;
    /// PCI device memory ranges.
    pub const PCI_RANGES: &[(usize, usize)] = &[];
    /// Timer interrupt num (PPI, physical timer).
    pub const TIMER_IRQ: usize = 0xf0;
    /// VirtIO MMIO regions with format (`base_paddr`, `size`).
    pub const VIRTIO_MMIO_REGIONS: &[(usize, usize)] = &[];

    /// IPI interrupt num (SGI, software generated interrupt).
    pub const IPI_IRQ: usize = 0x0;
}
///
/// Platform configs
///
pub mod plat {
    /// Number of CPUs.
    pub const CPU_NUM: usize = 16;
    /// Platform family (deprecated).
    pub const FAMILY: &str = "";
    /// Kernel address space base.
    pub const KERNEL_ASPACE_BASE: usize = 0xffff_8000_0000_0000;
    /// Kernel address space size.
    pub const KERNEL_ASPACE_SIZE: usize = 0x0000_7fff_ffff_f000;
    /// No need.
    pub const KERNEL_BASE_PADDR: usize = 0x20_0000;
    /// Base virtual address of the kernel image.
    pub const KERNEL_BASE_VADDR: usize = 0xffff_8000_0020_0000;
    /// Offset of bus address and phys address. some boards, the bus address is
    /// different from the physical address.
    pub const PHYS_BUS_OFFSET: usize = 0;
    /// No need.
    pub const PHYS_MEMORY_BASE: usize = 0;
    /// No need.
    pub const PHYS_MEMORY_SIZE: usize = 0x0;
    /// No need.
    pub const PHYS_VIRT_OFFSET: usize = 0;
}

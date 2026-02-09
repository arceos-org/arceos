//! Platform-specific constants and parameters for [ArceOS].
//!
//! Currently supported platform configs can be found in the [configs] directory of
//! the [ArceOS] root.
//!
//! [ArceOS]: https://github.com/arceos-org/arceos
//! [configs]: https://github.com/arceos-org/arceos/tree/main/configs

include!(concat!(env!("OUT_DIR"), "/dyn_impl_gen.rs"));

/// Architecture identifier.
pub const ARCH: &str = if cfg!(target_arch = "x86_64") {
    "x86_64"
} else if cfg!(target_arch = "aarch64") {
    "aarch64"
} else if cfg!(target_arch = "riscv64") {
    "riscv64"
} else if cfg!(target_arch = "loongarch64") {
    "loongarch64"
} else {
    "unknown"
};

/// Platform package.
pub const PACKAGE: &str = "axplat_dyn";
/// Platform identifier.
pub const PLATFORM: &str = "axplat_dyn";

/// Stack size of each task.
pub const TASK_STACK_SIZE: usize = _TASK_STACK_SIZE;

/// Number of timer ticks per second (Hz). A timer tick may contain several timer
/// interrupts.
pub const TICKS_PER_SEC: usize = 100;
///
/// Device specifications
///
pub mod devices {
    /// No need.
    pub const MMIO_REGIONS: &[(usize, usize)] = &[];
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
    pub const CPU_NUM: usize = super::_CPU_MAX_NUM;
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

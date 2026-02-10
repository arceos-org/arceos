// Copyright 2025 The Axvisor Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Platform-specific constants and parameters for [ArceOS].
//!
//! Currently supported platform configs can be found in the [configs] directory of
//! the [ArceOS] root.
//!
//! [ArceOS]: https://github.com/arceos-org/arceos
//! [configs]: https://github.com/arceos-org/arceos/tree/main/configs

#![no_std]

#[doc = " Architecture identifier."]
pub const ARCH: &str = "aarch64";
#[doc = " Platform package."]
pub const PACKAGE: &str = "axplat-aarch64-generic";
#[doc = " Platform identifier."]
pub const PLATFORM: &str = "aarch64-generic";
#[doc = " Stack size of each task."]
pub const TASK_STACK_SIZE: usize = 0x40000;
#[doc = " Number of timer ticks per second (Hz). A timer tick may contain several timer"]
#[doc = " interrupts."]
pub const TICKS_PER_SEC: usize = 100;
#[doc = ""]
#[doc = " Device specifications"]
#[doc = ""]
pub mod devices {
    #[doc = " MMIO regions with format (`base_paddr`, `size`)."]
    pub const MMIO_REGIONS: &[(usize, usize)] = &[
        (0xb000_0000, 0x1000_0000), // PCI config space
        (0xfe00_0000, 0xc0_0000),   // PCI devices
        (0xfec0_0000, 0x1000),      // IO APIC
        (0xfed0_0000, 0x1000),      // HPET
        (0xfee0_0000, 0x1000),      // Local APIC
    ];
    #[doc = " End PCI bus number."]
    pub const PCI_BUS_END: usize = 0xff;
    #[doc = " Base physical address of the PCIe ECAM space."]
    pub const PCI_ECAM_BASE: usize = 0xb000_0000;
    #[doc = " PCI device memory ranges."]
    pub const PCI_RANGES: &[(usize, usize)] = &[];
    #[doc = " Timer interrupt num (PPI, physical timer)."]
    pub const TIMER_IRQ: usize = 0xf0;
    #[doc = " VirtIO MMIO regions with format (`base_paddr`, `size`)."]
    pub const VIRTIO_MMIO_REGIONS: &[(usize, usize)] = &[];
}
#[doc = ""]
#[doc = " Platform configs"]
#[doc = ""]
pub mod plat {
    #[doc = " Number of CPUs."]
    pub const CPU_NUM: usize = 16;
    #[doc = " Platform family (deprecated)."]
    pub const FAMILY: &str = "";
    #[doc = " Kernel address space base."]
    pub const KERNEL_ASPACE_BASE: usize = 0xffff_8000_0000_0000;
    #[doc = " Kernel address space size."]
    pub const KERNEL_ASPACE_SIZE: usize = 0x0000_7fff_ffff_f000;
    #[doc = " No need."]
    pub const KERNEL_BASE_PADDR: usize = 0x20_0000;
    #[doc = " Base virtual address of the kernel image."]
    pub const KERNEL_BASE_VADDR: usize = 0xffff_8000_0020_0000;
    #[doc = " Offset of bus address and phys address. some boards, the bus address is"]
    #[doc = " different from the physical address."]
    pub const PHYS_BUS_OFFSET: usize = 0;
    #[doc = " No need."]
    pub const PHYS_MEMORY_BASE: usize = 0;
    #[doc = " No need."]
    pub const PHYS_MEMORY_SIZE: usize = 0x0;
    #[doc = " No need."]
    pub const PHYS_VIRT_OFFSET: usize = 0;
}

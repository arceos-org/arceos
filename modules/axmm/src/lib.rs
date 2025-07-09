//! [ArceOS](https://github.com/arceos-org/arceos) memory management module.

#![no_std]

#[macro_use]
extern crate log;
extern crate alloc;

mod aspace;
mod backend;

pub use self::aspace::AddrSpace;
pub use self::backend::Backend;

use axerrno::{AxError, AxResult};
use axhal::mem::{MemRegionFlags, phys_to_virt};
use axhal::paging::MappingFlags;
use kspin::SpinNoIrq;
use lazyinit::LazyInit;
use memory_addr::{MemoryAddr, PhysAddr, VirtAddr, va};
use memory_set::MappingError;

static KERNEL_ASPACE: LazyInit<SpinNoIrq<AddrSpace>> = LazyInit::new();

fn mapping_err_to_ax_err(err: MappingError) -> AxError {
    warn!("Mapping error: {:?}", err);
    match err {
        MappingError::InvalidParam => AxError::InvalidInput,
        MappingError::AlreadyExists => AxError::AlreadyExists,
        MappingError::BadState => AxError::BadState,
    }
}

fn reg_flag_to_map_flag(f: MemRegionFlags) -> MappingFlags {
    let mut ret = MappingFlags::empty();
    if f.contains(MemRegionFlags::READ) {
        ret |= MappingFlags::READ;
    }
    if f.contains(MemRegionFlags::WRITE) {
        ret |= MappingFlags::WRITE;
    }
    if f.contains(MemRegionFlags::EXECUTE) {
        ret |= MappingFlags::EXECUTE;
    }
    if f.contains(MemRegionFlags::DEVICE) {
        ret |= MappingFlags::DEVICE;
    }
    if f.contains(MemRegionFlags::UNCACHED) {
        ret |= MappingFlags::UNCACHED;
    }
    ret
}

/// Creates a new address space for kernel itself.
pub fn new_kernel_aspace() -> AxResult<AddrSpace> {
    let mut aspace = AddrSpace::new_empty(
        va!(axconfig::plat::KERNEL_ASPACE_BASE),
        axconfig::plat::KERNEL_ASPACE_SIZE,
    )?;
    for r in axhal::mem::memory_regions() {
        // mapped range should contain the whole region if it is not aligned.
        let start = r.paddr.align_down_4k();
        let end = (r.paddr + r.size).align_up_4k();
        aspace.map_linear(
            phys_to_virt(start),
            start,
            end - start,
            reg_flag_to_map_flag(r.flags),
        )?;
    }
    Ok(aspace)
}

/// Creates a new address space for user processes.
pub fn new_user_aspace(base: VirtAddr, size: usize) -> AxResult<AddrSpace> {
    let mut aspace = AddrSpace::new_empty(base, size)?;
    if !cfg!(target_arch = "aarch64") && !cfg!(target_arch = "loongarch64") {
        // ARMv8 (aarch64) and LoongArch64 use separate page tables for user space
        // (aarch64: TTBR0_EL1, LoongArch64: PGDL), so there is no need to copy the
        // kernel portion to the user page table.
        aspace.copy_mappings_from(&kernel_aspace().lock())?;
    }
    Ok(aspace)
}

/// Returns the globally unique kernel address space.
pub fn kernel_aspace() -> &'static SpinNoIrq<AddrSpace> {
    &KERNEL_ASPACE
}

/// Returns the root physical address of the kernel page table.
pub fn kernel_page_table_root() -> PhysAddr {
    KERNEL_ASPACE.lock().page_table_root()
}

/// Initializes virtual memory management.
///
/// It mainly sets up the kernel virtual memory address space and recreate a
/// fine-grained kernel page table.
pub fn init_memory_management() {
    info!("Initialize virtual memory management...");

    let kernel_aspace = new_kernel_aspace().expect("failed to initialize kernel address space");
    debug!("kernel address space init OK: {:#x?}", kernel_aspace);
    KERNEL_ASPACE.init_once(SpinNoIrq::new(kernel_aspace));
    unsafe { axhal::asm::write_kernel_page_table(kernel_page_table_root()) };
}

/// Initializes kernel paging for secondary CPUs.
pub fn init_memory_management_secondary() {
    unsafe { axhal::asm::write_kernel_page_table(kernel_page_table_root()) };
}

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
use axhal::mem::phys_to_virt;
use kspin::SpinNoIrq;
use lazyinit::LazyInit;
use memory_addr::{va, PhysAddr, VirtAddr};
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

/// Creates a new address space for kernel itself.
pub fn new_kernel_aspace() -> AxResult<AddrSpace> {
    let mut aspace = AddrSpace::new_empty(
        va!(axconfig::KERNEL_ASPACE_BASE),
        axconfig::KERNEL_ASPACE_SIZE,
    )?;
    for r in axhal::mem::memory_regions() {
        if r.size == 0 {
            info!("Skip zero-size memory region: {:?}", r);
            continue;
        }
        aspace.map_linear(phys_to_virt(r.paddr), r.paddr, r.size, r.flags.into())?;
    }
    Ok(aspace)
}

/// Creates a new address space for user processes.
pub fn new_user_aspace(base: VirtAddr, size: usize) -> AxResult<AddrSpace> {
    let mut aspace = AddrSpace::new_empty(base, size)?;
    if !cfg!(target_arch = "aarch64") {
        // ARMv8 use a separate page table (TTBR0_EL1) for user space, it
        // doesn't need to copy the kernel portion to the user page table.
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
    axhal::paging::set_kernel_page_table_root(kernel_page_table_root());
}

/// Initializes kernel paging for secondary CPUs.
pub fn init_memory_management_secondary() {
    axhal::paging::set_kernel_page_table_root(kernel_page_table_root());
}

//! Physical memory management.

use heapless::Vec;
use lazyinit::LazyInit;

use axplat::mem::{check_sorted_ranges_overlap, ranges_difference};

pub use axplat::mem::{MemRegionFlags, PhysMemRegion};
pub use axplat::mem::{
    mmio_ranges, phys_ram_ranges, phys_to_virt, reserved_phys_ram_ranges, total_ram_size,
    virt_to_phys,
};
pub use memory_addr::{PAGE_SIZE_4K, PhysAddr, PhysAddrRange, VirtAddr, VirtAddrRange, pa, va};

const MAX_REGIONS: usize = 128;

static ALL_MEM_REGIONS: LazyInit<Vec<PhysMemRegion, MAX_REGIONS>> = LazyInit::new();

/// Returns an iterator over all physical memory regions.
pub fn memory_regions() -> impl Iterator<Item = PhysMemRegion> {
    ALL_MEM_REGIONS.iter().cloned()
}

/// Fills the `.bss` section with zeros.
///
/// It requires the symbols `_sbss` and `_ebss` to be defined in the linker script.
///
/// # Safety
///
/// This function is unsafe because it writes `.bss` section directly.
pub unsafe fn clear_bss() {
    unsafe {
        core::slice::from_raw_parts_mut(_sbss as usize as *mut u8, _ebss as usize - _sbss as usize)
            .fill(0);
    }
}

/// Initializes physical memory regions.
pub fn init() {
    let mut all_regions = Vec::new();
    let mut push = |r: PhysMemRegion| {
        if r.size > 0 {
            all_regions.push(r).expect("too many memory regions");
        }
    };

    // Push regions in kernel image
    push(PhysMemRegion {
        paddr: virt_to_phys((_stext as usize).into()),
        size: _etext as usize - _stext as usize,
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::EXECUTE,
        name: ".text",
    });
    push(PhysMemRegion {
        paddr: virt_to_phys((_srodata as usize).into()),
        size: _erodata as usize - _srodata as usize,
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ,
        name: ".rodata",
    });
    push(PhysMemRegion {
        paddr: virt_to_phys((_sdata as usize).into()),
        size: _edata as usize - _sdata as usize,
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
        name: ".data .tdata .tbss .percpu",
    });
    push(PhysMemRegion {
        paddr: virt_to_phys((boot_stack as usize).into()),
        size: boot_stack_top as usize - boot_stack as usize,
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
        name: "boot stack",
    });
    push(PhysMemRegion {
        paddr: virt_to_phys((_sbss as usize).into()),
        size: _ebss as usize - _sbss as usize,
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
        name: ".bss",
    });

    // Push MMIO & reserved regions
    for &(start, size) in mmio_ranges() {
        push(PhysMemRegion::new_mmio(start, size, "mmio"));
    }
    for &(start, size) in reserved_phys_ram_ranges() {
        push(PhysMemRegion::new_reserved(start, size, "reserved"));
    }

    // Combine kernel image range and reserved ranges
    let kernel_start = virt_to_phys(va!(_skernel as usize)).as_usize();
    let kernel_size = _ekernel as usize - _skernel as usize;
    let mut reserved_ranges = reserved_phys_ram_ranges()
        .iter()
        .cloned()
        .chain(core::iter::once((kernel_start, kernel_size))) // kernel image range is also reserved
        .collect::<Vec<_, MAX_REGIONS>>();

    // Remove all reserved ranges from RAM ranges, and push the remaining as free memory
    reserved_ranges.sort_unstable_by_key(|&(start, _size)| start);
    ranges_difference(phys_ram_ranges(), &reserved_ranges, |(start, size)| {
        push(PhysMemRegion::new_ram(start, size, "free memory"));
    })
    .inspect_err(|(a, b)| error!("Reserved memory region {:#x?} overlaps with {:#x?}", a, b))
    .unwrap();

    // Check overlapping
    all_regions.sort_unstable_by_key(|r| r.paddr);
    check_sorted_ranges_overlap(all_regions.iter().map(|r| (r.paddr.into(), r.size)))
        .inspect_err(|(a, b)| error!("Physical memory region {:#x?} overlaps with {:#x?}", a, b))
        .unwrap();

    ALL_MEM_REGIONS.init_once(all_regions);
}

unsafe extern "C" {
    fn _stext();
    fn _etext();
    fn _srodata();
    fn _erodata();
    fn _sdata();
    fn _edata();
    fn _sbss();
    fn _ebss();
    fn _skernel();
    fn _ekernel();
    fn boot_stack();
    fn boot_stack_top();
}

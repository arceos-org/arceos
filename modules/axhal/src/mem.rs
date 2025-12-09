//! Physical memory management.

use heapless::Vec;
use lazyinit::LazyInit;

use axplat::mem::{check_sorted_ranges_overlap, ranges_difference};

pub use axplat::mem::{MemRegionFlags, PhysMemRegion};
pub use axplat::mem::{
    kernel_aspace, mmio_ranges, phys_ram_ranges, phys_to_virt, reserved_phys_ram_ranges,
    total_ram_size, virt_to_phys,
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
        let sbss = _sbss as *const () as usize;
        let ebss = _ebss as *const () as usize;
        core::slice::from_raw_parts_mut(sbss as *mut u8, ebss - sbss).fill(0);
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

    let stext = _stext as *const() as usize;
    let etext = _etext as *const() as usize;
    // Push regions in kernel image
    push(PhysMemRegion {
        paddr: virt_to_phys((stext).into()),
        size: etext - stext,
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::EXECUTE,
        name: ".text",
    });
    let srodata = _srodata as *const() as usize;
    let erodata = _erodata as *const() as usize;
    push(PhysMemRegion {
        paddr: virt_to_phys((srodata).into()),
        size: erodata - srodata,
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ,
        name: ".rodata",
    });
    let sdata = _sdata as *const() as usize;
    let edata = _edata as *const() as usize;
    push(PhysMemRegion {
        paddr: virt_to_phys((sdata).into()),
        size: edata - sdata,
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
        name: ".data .tdata .tbss .percpu",
    });
    let boot_stack = boot_stack as *const() as usize;
    let boot_stack_top = boot_stack_top as *const() as usize;
    push(PhysMemRegion {
        paddr: virt_to_phys((boot_stack).into()),
        size: boot_stack_top - boot_stack,
        flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
        name: "boot stack",
    });
    let sbss = _sbss as *const() as usize;
    let ebss = _ebss as *const() as usize;
    push(PhysMemRegion {
        paddr: virt_to_phys((sbss).into()),
        size: ebss - sbss,
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
    let skernel = _skernel as *const() as usize;
    let ekernel = _ekernel as *const() as usize;
    let kernel_start = virt_to_phys(va!(skernel)).as_usize();
    let kernel_size = ekernel - skernel;
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
    .inspect_err(|(a, b)| error!("Reserved memory region {a:#x?} overlaps with {b:#x?}"))
    .unwrap();

    // Check overlapping
    all_regions.sort_unstable_by_key(|r| r.paddr);
    check_sorted_ranges_overlap(all_regions.iter().map(|r| (r.paddr.into(), r.size)))
        .inspect_err(|(a, b)| error!("Physical memory region {a:#x?} overlaps with {b:#x?}"))
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

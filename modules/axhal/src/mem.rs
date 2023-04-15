use core::fmt;

#[doc(no_inline)]
pub use memory_addr::{PhysAddr, VirtAddr, PAGE_SIZE_4K};

bitflags::bitflags! {
    pub struct MemRegionFlags: usize {
        const READ          = 1 << 0;
        const WRITE         = 1 << 1;
        const EXECUTE       = 1 << 2;
        const DEVICE        = 1 << 4;
        const RESERVED      = 1 << 5;
        const FREE          = 1 << 6;
    }
}

impl fmt::Debug for MemRegionFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

#[derive(Debug)]
pub struct MemRegion {
    pub paddr: PhysAddr,
    pub size: usize,
    pub flags: MemRegionFlags,
    pub name: &'static str,
}

struct MemRegionIter {
    idx: usize,
}

impl Iterator for MemRegionIter {
    type Item = MemRegion;

    fn next(&mut self) -> Option<Self::Item> {
        use crate::platform::mem::{memory_region_at, memory_regions_num};
        let ret = if self.idx < memory_regions_num() {
            memory_region_at(self.idx)
        } else {
            None
        };
        self.idx += 1;
        ret
    }
}

#[inline]
pub const fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    PhysAddr::from(vaddr.as_usize() - axconfig::PHYS_VIRT_OFFSET)
}

#[inline]
pub const fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    VirtAddr::from(paddr.as_usize() + axconfig::PHYS_VIRT_OFFSET)
}

pub fn memory_regions() -> impl Iterator<Item = MemRegion> {
    MemRegionIter { idx: 0 }
}

#[allow(dead_code)]
pub(crate) const fn common_memory_regions_num() -> usize {
    6 + axconfig::MMIO_REGIONS.len()
}

#[allow(dead_code)]
pub(crate) fn common_memory_region_at(idx: usize) -> Option<MemRegion> {
    let mmio_regions = axconfig::MMIO_REGIONS;
    let r = match idx {
        0 => MemRegion {
            paddr: virt_to_phys((stext as usize).into()),
            size: etext as usize - stext as usize,
            flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::EXECUTE,
            name: ".text",
        },
        1 => MemRegion {
            paddr: virt_to_phys((srodata as usize).into()),
            size: erodata as usize - srodata as usize,
            flags: MemRegionFlags::RESERVED | MemRegionFlags::READ,
            name: ".rodata",
        },
        2 => MemRegion {
            paddr: virt_to_phys((sdata as usize).into()),
            size: edata as usize - sdata as usize,
            flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
            name: ".data",
        },
        3 => MemRegion {
            paddr: virt_to_phys((percpu_start as usize).into()),
            size: percpu_end as usize - percpu_start as usize,
            flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
            name: ".percpu",
        },
        4 => MemRegion {
            paddr: virt_to_phys((boot_stack as usize).into()),
            size: boot_stack_top as usize - boot_stack as usize,
            flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
            name: "boot stack",
        },
        5 => MemRegion {
            paddr: virt_to_phys((sbss as usize).into()),
            size: ebss as usize - sbss as usize,
            flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
            name: ".bss",
        },
        i if i < 6 + mmio_regions.len() => MemRegion {
            paddr: mmio_regions[i - 6].0.into(),
            size: mmio_regions[i - 6].1,
            flags: MemRegionFlags::RESERVED
                | MemRegionFlags::DEVICE
                | MemRegionFlags::READ
                | MemRegionFlags::WRITE,
            name: "mmio",
        },
        _ => return None,
    };
    Some(r)
}

#[allow(dead_code)]
pub(crate) fn clear_bss() {
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss();
    fn ebss();
    fn boot_stack();
    fn boot_stack_top();
    fn percpu_start();
    fn percpu_end();
}

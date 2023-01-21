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

#[derive(Debug)]
pub struct MemRegion {
    pub paddr: usize,
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

pub const fn virt_to_phys(vaddr: usize) -> usize {
    vaddr - axconfig::PHYS_VIRT_OFFSET
}

pub const fn phys_to_virt(paddr: usize) -> usize {
    paddr + axconfig::PHYS_VIRT_OFFSET
}

#[allow(dead_code)]
pub(crate) const fn common_memory_regions_num() -> usize {
    5 + axconfig::MMIO_REGIONS.len()
}

#[allow(dead_code)]
pub(crate) fn common_memory_region_at(idx: usize) -> Option<MemRegion> {
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
    }

    let mmio_regions = axconfig::MMIO_REGIONS;
    let r = match idx {
        0 => MemRegion {
            paddr: virt_to_phys(stext as usize),
            size: etext as usize - stext as usize,
            flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::EXECUTE,
            name: "text",
        },
        1 => MemRegion {
            paddr: virt_to_phys(srodata as usize),
            size: erodata as usize - srodata as usize,
            flags: MemRegionFlags::RESERVED | MemRegionFlags::READ,
            name: "rodata",
        },
        2 => MemRegion {
            paddr: virt_to_phys(sdata as usize),
            size: edata as usize - sdata as usize,
            flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
            name: "data",
        },
        3 => MemRegion {
            paddr: virt_to_phys(sbss as usize),
            size: ebss as usize - sbss as usize,
            flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
            name: "bss",
        },
        4 => MemRegion {
            paddr: virt_to_phys(boot_stack as usize),
            size: boot_stack_top as usize - boot_stack as usize,
            flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
            name: "boot stack",
        },
        i if i < 5 + mmio_regions.len() => MemRegion {
            paddr: mmio_regions[i - 5].0,
            size: mmio_regions[i - 5].1,
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

pub fn memory_regions() -> impl Iterator<Item = MemRegion> {
    MemRegionIter { idx: 0 }
}

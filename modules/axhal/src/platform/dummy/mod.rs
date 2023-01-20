#![allow(unused_variables)]

pub mod console {
    pub fn putchar(c: u8) {
        unimplemented!()
    }

    pub fn getchar() -> Option<u8> {
        unimplemented!()
    }
}

pub mod misc {
    pub fn terminate() -> ! {
        unimplemented!()
    }
}

pub mod mem {
    pub use crate::common::mem::{memory_regions, phys_to_virt, virt_to_phys};
    pub use crate::common::mem::{MemRegion, MemRegionFlags};

    pub(crate) fn memory_regions_num() -> usize {
        0
    }

    pub(crate) fn memory_region_at(idx: usize) -> Option<MemRegion> {
        None
    }
}

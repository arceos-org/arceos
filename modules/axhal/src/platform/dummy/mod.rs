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
    pub(crate) fn memory_regions_num() -> usize {
        0
    }

    pub(crate) fn memory_region_at(idx: usize) -> Option<crate::mem::MemRegion> {
        None
    }
}

pub mod time {
    pub fn current_ticks() -> u64 {
        0
    }

    pub fn ticks_to_nanos(ticks: u64) -> u64 {
        ticks
    }
}

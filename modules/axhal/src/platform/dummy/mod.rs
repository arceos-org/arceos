#![allow(unused_variables)]
#![allow(dead_code)]

pub(crate) fn handle_irq(_irq_num: usize) {}

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

#[cfg(feature = "smp")]
pub mod mp {
    pub fn start_secondary_cpu(
        hardid: usize,
        entry: crate::mem::PhysAddr,
        stack_top: crate::mem::PhysAddr,
    ) {
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
    pub const TIMER_IRQ_NUM: usize = 0;

    pub fn current_ticks() -> u64 {
        0
    }

    pub fn ticks_to_nanos(ticks: u64) -> u64 {
        ticks
    }

    pub fn nanos_to_ticks(nanos: u64) -> u64 {
        nanos
    }

    pub fn set_oneshot_timer(deadline_ns: u64) {}
}

pub mod irq {
    pub const MAX_IRQ_COUNT: usize = 256;

    pub fn set_enable(irq_num: usize, enabled: bool) {}

    pub fn register_handler(irq_num: usize, handler: crate::irq::IrqHandler) -> bool {
        false
    }

    pub fn dispatch_irq(irq_num: usize) {}
}

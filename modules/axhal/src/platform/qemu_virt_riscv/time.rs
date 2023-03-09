use riscv::register::{sie, time};

const NANOS_PER_TICK: u64 = crate::time::NANOS_PER_SEC / axconfig::TIMER_FREQUENCY as u64;

pub const TIMER_IRQ_NUM: usize = super::irq::S_TIMER;

#[inline]
pub fn current_ticks() -> u64 {
    time::read() as u64
}

#[inline]
pub const fn ticks_to_nanos(ticks: u64) -> u64 {
    ticks * NANOS_PER_TICK
}

#[inline]
pub const fn nanos_to_ticks(nanos: u64) -> u64 {
    nanos / NANOS_PER_TICK
}

pub fn set_oneshot_timer(deadline_ns: u64) {
    sbi_rt::set_timer(nanos_to_ticks(deadline_ns));
}

pub fn init() {
    unsafe {
        sie::set_ssoft();
        sie::set_stimer();
        sie::set_sext();
    }
    sbi_rt::set_timer(0);
}

use riscv::register::time;

const NANOS_PER_TICK: u64 = crate::time::NANOS_PER_SEC / axconfig::TIMER_FREQUENCY as u64;

pub fn current_ticks() -> u64 {
    time::read() as u64
}

pub const fn ticks_to_nanos(ticks: u64) -> u64 {
    ticks * NANOS_PER_TICK
}

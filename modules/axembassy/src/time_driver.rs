use core::cell::RefCell;
use core::task;

use axhal::irq;
use axhal::time::{self, NANOS_PER_SEC, TIMER_IRQ_NUM};
use kspin::SpinNoIrq;

use embassy_time_driver::Driver;
use embassy_time_driver::TICK_HZ;
use embassy_time_queue_utils::Queue;

pub fn ticks_to_nanos(ticks: u64) -> u64 {
    (ticks as u128 * NANOS_PER_SEC as u128 / TICK_HZ as u128) as u64
}

pub fn nanos_to_ticks(nanos: u64) -> u64 {
    (nanos as u128 * TICK_HZ as u128 / NANOS_PER_SEC as u128) as u64
}

pub struct AxDriver {
    queue: SpinNoIrq<RefCell<Queue>>,
    periodic_interval_nanos: SpinNoIrq<u64>,
}

impl AxDriver {
    pub const fn new() -> Self {
        AxDriver {
            queue: SpinNoIrq::new(RefCell::new(Queue::new())),
            periodic_interval_nanos: SpinNoIrq::new(0),
        }
    }

    pub fn runtime_init(&self, periodic_interval_nanos: u64, sched_lock: &SpinNoIrq<u64>) {
        let mut _queue_guard = self.queue.lock();
        let mut sched_lock = sched_lock.lock();
        let mut interval_lock = self.periodic_interval_nanos.lock();

        let now_nanos = axhal::time::monotonic_time_nanos();

        *interval_lock = periodic_interval_nanos;
        *sched_lock = now_nanos + periodic_interval_nanos;
    }

    /// Set the alarm to wake up at the given time in **nanosecond**.
    fn set_alarm_at(&self, at: u64) -> bool {
        if at == u64::MAX {
            // TODO: Disable the hardware timer interrupt here, as there's nothing to wait for.
            // Assuming time::disable_timer_interrupt() exists or similar via axhal/platform.
            irq::set_enable(TIMER_IRQ_NUM, false);
            return true;
        }

        let nanos_now = time::monotonic_time_nanos();
        if at <= nanos_now {
            return false;
        }

        time::set_oneshot_timer(at);
        true
    }
}

impl Driver for AxDriver {
    // Returns the current time in **embassy ticks**.
    fn now(&self) -> u64 {
        let nanos_now = time::monotonic_time_nanos();

        nanos_to_ticks(nanos_now)
    }

    // Driver::schedule_wake() will add to the queue, then call set_alarm_nanos_locked
    // for the earliest of the next embassy timer or next scheduler tick.
    fn schedule_wake(&self, at: u64, waker: &task::Waker) {
        let queue_guard = self.queue.lock();
        let mut queue = queue_guard.borrow_mut();

        if queue.schedule_wake(at, waker) {
            let mut next_at = queue.next_expiration(self.now());
            // repeatly set until it succeeds
            while next_at != u64::MAX {
                if self.set_alarm_at(next_at) {
                    break;
                }
                next_at = queue.next_expiration(self.now())
            }
        }
    }
}
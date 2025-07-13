use core::cell::RefCell;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task;

use axhal::time::{self, NANOS_PER_SEC, set_oneshot_timer};
use kspin::SpinNoIrq;

use embassy_time_driver::TICK_HZ;
use embassy_time_driver::{Driver, time_driver_impl};
use embassy_time_queue_utils::Queue;

/// Manipulation of Global `AxDriver`
pub struct AxDriverAPI;

impl AxDriverAPI {
    /// Dequeue expired timer and return nanos of next expiration
    pub fn next_expiration(period: u64) -> u64 {
        AX_DRIVER.next_expiration(period)
    }
}

fn ticks_to_nanos(ticks: u64) -> u64 {
    (ticks as u128 * NANOS_PER_SEC as u128 / TICK_HZ as u128) as u64
}

fn nanos_to_ticks(nanos: u64) -> u64 {
    (nanos as u128 * TICK_HZ as u128 / NANOS_PER_SEC as u128) as u64
}

struct AxDriver {
    queue: SpinNoIrq<RefCell<Queue>>,
    // static period interval
    period_nanos: AtomicU64,
}

time_driver_impl!(static AX_DRIVER: AxDriver = AxDriver::new());

impl AxDriver {
    pub const fn new() -> Self {
        AxDriver {
            queue: SpinNoIrq::new(RefCell::new(Queue::new())),
            period_nanos: AtomicU64::new(0),
        }
    }

    pub fn nanos_now() -> u64 {
        time::monotonic_time_nanos()
    }

    pub fn ticks_now() -> u64 {
        let nanos_now = time::monotonic_time_nanos();
        nanos_to_ticks(nanos_now)
    }

    /// schedule waker and set timer only if the next expiration interval is shorter than the periodic interval
    pub fn schedule_wake(&self, at: u64, waker: &task::Waker) {
        let queue_guard = self.queue.lock();
        let mut queue = queue_guard.borrow_mut();

        if queue.schedule_wake(at, waker) {
            let ticks_next_at = queue.next_expiration(self.now());
            let nanos_next_at = ticks_to_nanos(ticks_next_at);
            let nanos_next_interval = nanos_next_at - Self::nanos_now();
            let nanos_period = self.period_nanos.load(Ordering::Relaxed);
            if nanos_next_interval < nanos_period {
                // only set timer if it is less than the periodic interval
                set_oneshot_timer(nanos_next_at);
            }
        }
    }

    /// Dequeue expired timer and return nanos of next expiration
    pub fn next_expiration(&self, period: u64) -> u64 {
        let queue_guard = self.queue.lock();
        let mut queue = queue_guard.borrow_mut();
        self.period_nanos.store(period, Ordering::Release);

        let ticks_now = self.now();

        let ticks_next_expired = queue.next_expiration(ticks_now);
        let nanos_next_expired = ticks_to_nanos(ticks_next_expired);
        nanos_next_expired
    }
}

impl Driver for AxDriver {
    // Returns the current time in **embassy ticks**.
    fn now(&self) -> u64 {
        Self::ticks_now()
    }

    /// Schedule to wake up task by **embassy ticks**
    ///
    /// Set timer only if the newest expiration interval is before the period interval
    fn schedule_wake(&self, at: u64, waker: &task::Waker) {
        self.schedule_wake(at, waker);
    }
}

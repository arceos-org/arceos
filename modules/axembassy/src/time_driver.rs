use core::cell::RefCell;
use core::task;

use axhal::irq;
use axhal::time::{self, NANOS_PER_SEC, TIMER_IRQ_NUM};
use kspin::SpinNoIrq;

use embassy_time_driver::TICK_HZ;
use embassy_time_driver::{Driver, time_driver_impl};
use embassy_time_queue_utils::Queue;

pub struct AxDriverAPI;

impl AxDriverAPI {
    pub fn runtime_init(periodic_interval_nanos: u64) {
        AX_DRIVER.runtime_init(periodic_interval_nanos, unsafe { NANOS_NEXT_SCHED.current_ref_raw() });
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
    periodic_interval_nanos: SpinNoIrq<u64>,
}

time_driver_impl!(static AX_DRIVER: AxDriver = AxDriver::new());

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

#[percpu::def_percpu]
static NANOS_NEXT_SCHED: SpinNoIrq<u64> = SpinNoIrq::new(0);

/// Timer interrupt handler merged with embassy
///
/// Integrate both embassy and task scheduler:
/// - Record schedule time to call `axtask::on_timer_tick()`
/// - Set the next timer alarm by comparing the next embassy timer and the next scheduler tick repeatedly until works.
pub fn embassy_update_timer() {
    let queue_guard = AX_DRIVER.queue.lock();
    let mut queue = queue_guard.borrow_mut();

    let ticks_now = AX_DRIVER.now();
    let nanos_now = time::monotonic_time_nanos();

    let ticks_next_at = queue.next_expiration(ticks_now);
    drop(queue);

    let mut sched_lock = unsafe { NANOS_NEXT_SCHED.current_ref_mut_raw().lock() };
    let periodic_lock = AX_DRIVER.periodic_interval_nanos.lock();

    let mut nanos_next_sched = *sched_lock;
    let schedule_tick = if nanos_now >= nanos_next_sched {
        #[cfg(feature = "multitask")]
        axtask::on_timer_tick();

        let periodic = *periodic_lock;
        while nanos_next_sched <= nanos_now {
            nanos_next_sched += periodic;
        }
        *sched_lock = nanos_next_sched;
        true
    } else {
        false
    };

    let (timer_tick, nanos_next_at) = if ticks_next_at == u64::MAX {
        (false, u64::MAX)
    } else {
        (true, ticks_to_nanos(ticks_next_at))
    };

    let nanos_earlier = core::cmp::min(nanos_next_sched, nanos_next_at);
    let mut nanos_try = nanos_earlier;
    while nanos_try != u64::MAX {
        if AX_DRIVER.set_alarm_at(nanos_try) {
            break;
        }

        // Setting failed
        let ticks_next_failure = queue_guard.borrow_mut().next_expiration(AX_DRIVER.now());
        let nanos_next_failure = if ticks_next_failure == u64::MAX {
            u64::MAX
        } else {
            ticks_to_nanos(ticks_next_failure)
        };

        let sched_failure_lock = unsafe { NANOS_NEXT_SCHED.current_ref_raw().lock() };
        let nanos_sched_failure = *sched_failure_lock;
        nanos_try = core::cmp::min(nanos_next_failure, nanos_sched_failure);
    };
    
    if timer_tick || schedule_tick {
        #[cfg(feature="executor")]
        {
            use crate::executor::signal_executor;
            signal_executor();
        }
    }
}
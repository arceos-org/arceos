#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::{boxed::Box, collections::BinaryHeap};
use core::cmp::{Ord, Ordering, PartialOrd};
use core::time::Duration;

pub type TimeValue = Duration;

pub trait TimerEvent {
    fn callback(self, now: TimeValue);
}

struct TimerEventWrapper<E> {
    deadline: TimeValue,
    event: E,
}

pub struct TimerList<E: TimerEvent> {
    events: BinaryHeap<TimerEventWrapper<E>>,
}

impl<E> PartialOrd for TimerEventWrapper<E> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.deadline.partial_cmp(&self.deadline) // reverse ordering for Min-heap
    }
}

impl<E> Ord for TimerEventWrapper<E> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.deadline.cmp(&self.deadline) // reverse ordering for Min-heap
    }
}

impl<E> PartialEq for TimerEventWrapper<E> {
    fn eq(&self, other: &Self) -> bool {
        self.deadline.eq(&other.deadline)
    }
}

impl<E> Eq for TimerEventWrapper<E> {}

impl<E: TimerEvent> TimerList<E> {
    pub fn new() -> Self {
        Self {
            events: BinaryHeap::new(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn set(&mut self, deadline: TimeValue, event: E) {
        self.events.push(TimerEventWrapper { deadline, event });
    }

    /// Cancel all events that meet the condition.
    ///
    /// TODO: performance optimization
    pub fn cancel<F>(&mut self, condition: F)
    where
        F: Fn(&E) -> bool,
    {
        self.events.retain(|e| !condition(&e.event));
    }

    #[inline]
    pub fn next_deadline(&self) -> Option<TimeValue> {
        self.events.peek().map(|e| e.deadline)
    }

    pub fn expire_one(&mut self, now: TimeValue) -> Option<(TimeValue, E)> {
        if let Some(e) = self.events.peek() {
            if e.deadline <= now {
                return self.events.pop().map(|e| (e.deadline, e.event));
            }
        }
        None
    }
}

impl<E: TimerEvent> Default for TimerList<E> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TimerEventFn(Box<dyn FnOnce(TimeValue) + 'static>);

impl TimerEventFn {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(TimeValue) + 'static,
    {
        Self(Box::new(f))
    }
}

impl TimerEvent for TimerEventFn {
    fn callback(self, now: TimeValue) {
        (self.0)(now)
    }
}

#[cfg(test)]
mod tests {
    use super::{TimeValue, TimerEvent, TimerEventFn, TimerList};
    use core::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    #[test]
    fn test_timer_list() {
        const EVENT_ORDER: &[usize; 4] = &[1, 4, 3, 0]; // timer 2 was canceled
        static COUNT: AtomicUsize = AtomicUsize::new(0);

        struct TestTimerEvent(usize, TimeValue);

        impl TimerEvent for TestTimerEvent {
            fn callback(self, now: TimeValue) {
                let idx = COUNT.fetch_add(1, Ordering::SeqCst);
                assert_eq!(self.0, EVENT_ORDER[idx]);
                println!(
                    "timer {} expired at {:?}, delta = {:?}",
                    self.0,
                    now,
                    now - self.1
                );
            }
        }

        let mut timer_list = TimerList::new();
        let start_time = Instant::now();
        let deadlines = [
            Duration::new(3, 0),            // 3.0 sec
            Duration::from_micros(500_000), // 0.5 sec
            Duration::from_secs(4),         // 4.0 sec, canceled
            Duration::new(2, 990_000_000),  // 2.99 sec
            Duration::from_millis(1000),    // 1.0 sec,
        ];

        for (i, &ddl) in deadlines.iter().enumerate() {
            timer_list.set(ddl, TestTimerEvent(i, ddl));
        }

        while !timer_list.is_empty() {
            let now = Instant::now().duration_since(start_time);
            if now.as_secs() > 3 {
                timer_list.cancel(|e| e.0 == 2);
            }
            while let Some((_deadline, event)) = timer_list.expire_one(now) {
                event.callback(now);
            }
            std::thread::sleep(Duration::from_millis(10)); // sleep 10 ms
        }

        assert_eq!(COUNT.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn test_timer_list_fn() {
        let mut timer_list = TimerList::new();
        let start_time = Instant::now();
        let deadlines = [
            Duration::new(1, 1_000_000),    // 1.001 sec
            Duration::from_micros(750_000), // 0.75 sec
        ];

        for ddl in deadlines {
            timer_list.set(
                ddl,
                TimerEventFn::new(|now| println!("timer fn expired at {:?}", now)),
            );
        }

        while !timer_list.is_empty() {
            let now = Instant::now().duration_since(start_time);
            while let Some((_deadline, event)) = timer_list.expire_one(now) {
                event.callback(now);
            }
        }
    }
}

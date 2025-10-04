use alloc::{boxed::Box, vec::Vec};
use core::sync::atomic::{AtomicU64, Ordering};

use axhal::time::{TimeValue, wall_time};
use kernel_guard::{NoOp, NoPreemptIrqSave};
use kspin::{SpinNoIrq, SpinRaw};
use weak_map::WeakMap;

use crate::{AxTaskRef, WeakAxTaskRef, select_run_queue};

static TIMER_CALLBACKS: SpinRaw<Vec<Box<dyn Fn(TimeValue) + Send + Sync>>> =
    SpinRaw::new(Vec::new());

pub fn register_timer_callback<F>(callback: F)
where
    F: Fn(TimeValue) + Send + Sync + 'static,
{
    let _g = NoPreemptIrqSave::new();
    TIMER_CALLBACKS.lock().push(Box::new(callback));
}

static TIMER_KEY: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TimerKey {
    deadline: TimeValue,
    key: u64,
}

static TIMER_WHEEL: SpinNoIrq<WeakMap<TimerKey, WeakAxTaskRef>> = SpinNoIrq::new(WeakMap::new());

pub(crate) fn set_timer(deadline: TimeValue, task: &AxTaskRef) -> Option<TimerKey> {
    if deadline <= wall_time() {
        return None;
    }

    let mut wheel = TIMER_WHEEL.lock();
    let key = TimerKey {
        deadline,
        key: TIMER_KEY.fetch_add(1, Ordering::AcqRel),
    };
    wheel.insert(key, task);

    Some(key)
}

pub(crate) fn cancel_timer(key: &TimerKey) {
    let mut wheel = TIMER_WHEEL.lock();
    wheel.remove(key);
}

pub(crate) fn has_timer(key: &TimerKey) -> bool {
    TIMER_WHEEL.lock().contains_key(key)
}

pub(crate) fn check_events() {
    for callback in TIMER_CALLBACKS.lock().iter() {
        callback(wall_time());
    }

    let mut wheel = TIMER_WHEEL.lock();
    for (key, maybe_task) in &mut *wheel {
        if key.deadline <= wall_time() {
            if let Some(task) = maybe_task.upgrade() {
                select_run_queue::<NoOp>(&task).unblock_task(task, true);
                core::mem::take(maybe_task);
            }
        } else {
            break;
        }
    }
}

use alloc::{boxed::Box, vec::Vec};
use core::sync::atomic::{AtomicU64, Ordering};

use axhal::time::{TimeValue, wall_time};
use kernel_guard::{NoOp, NoPreemptIrqSave};
use timer_list::{TimerEvent, TimerList};

use crate::{AxTaskRef, select_run_queue};

static TIMER_TICKET_ID: AtomicU64 = AtomicU64::new(1);

percpu_static! {
    TIMER_LIST: TimerList<TaskWakeupEvent> = TimerList::new(),
    TIMER_CALLBACKS: Vec<Box<dyn Fn(TimeValue) + Send + Sync>> = Vec::new(),
}

struct TaskWakeupEvent {
    ticket_id: u64,
    task: AxTaskRef,
}

impl TimerEvent for TaskWakeupEvent {
    fn callback(self, _now: TimeValue) {
        // Ignore the timer event if timeout was set but not triggered
        // (wake up by `WaitQueue::notify()`).
        // Judge if this timer event is still valid by checking the ticket ID.
        if self.task.timer_ticket() != self.ticket_id {
            // Timer ticket ID is not matched.
            // Just ignore this timer event and return.
            return;
        }

        // Timer ticket match.
        select_run_queue::<NoOp>(&self.task).unblock_task(self.task, true)
    }
}

/// Registers a callback function to be called on each timer tick.
pub fn register_timer_callback<F>(callback: F)
where
    F: Fn(TimeValue) + Send + Sync + 'static,
{
    let _g = NoPreemptIrqSave::new();
    unsafe {
        TIMER_CALLBACKS
            .current_ref_mut_raw()
            .push(Box::new(callback))
    };
}

fn check_callbacks() {
    for callback in unsafe { TIMER_CALLBACKS.current_ref_raw().iter() } {
        callback(wall_time());
    }
}

pub(crate) fn set_alarm_wakeup(deadline: TimeValue, task: AxTaskRef) {
    let _g = NoPreemptIrqSave::new();
    TIMER_LIST.with_current(|timer_list| {
        let ticket_id = TIMER_TICKET_ID.fetch_add(1, Ordering::AcqRel);
        task.set_timer_ticket(ticket_id);
        timer_list.set(deadline, TaskWakeupEvent { ticket_id, task });
    })
}

// SAFETY: only called in timer irq handler, so irq and preemption are
// both disabled here.
pub fn check_events() {
    check_callbacks();
    loop {
        let now = wall_time();
        let event = unsafe {
            // Safety: IRQs are disabled at this time.
            TIMER_LIST.current_ref_mut_raw()
        }
        .expire_one(now);
        if let Some((_deadline, event)) = event {
            event.callback(now);
        } else {
            break;
        }
    }

    // Handle async timer events
    crate::future::check_timer_events();
}

use alloc::sync::Arc;

use lazyinit::LazyInit;
use timer_list::{TimeValue, TimerEvent, TimerList};

use axhal::time::wall_time;

use crate::{select_run_queue, AxTaskRef};

percpu_static! {
    TIMER_LIST: LazyInit<TimerList<TaskWakeupEvent>> = LazyInit::new(),
}

struct TaskWakeupEvent(AxTaskRef);

impl TimerEvent for TaskWakeupEvent {
    fn callback(self, _now: TimeValue) {
        self.0.set_in_timer_list(false);
        select_run_queue(
            #[cfg(feature = "smp")]
            self.0.clone(),
        )
        .unblock_task(self.0, true)
    }
}

pub fn set_alarm_wakeup(deadline: TimeValue, task: AxTaskRef) {
    TIMER_LIST.with_current(|timer_list| {
        task.set_in_timer_list(true);
        timer_list.set(deadline, TaskWakeupEvent(task));
    })
}

pub fn cancel_alarm(task: &AxTaskRef) {
    TIMER_LIST.with_current(|timer_list| {
        task.set_in_timer_list(false);
        timer_list.cancel(|t| Arc::ptr_eq(&t.0, task));
    })
}

pub fn check_events() {
    loop {
        let now = wall_time();
        let event = TIMER_LIST.with_current(|timer_list| timer_list.expire_one(now));
        if let Some((_deadline, event)) = event {
            event.callback(now);
        } else {
            break;
        }
    }
}

pub fn init() {
    TIMER_LIST.with_current(|timer_list| {
        timer_list.init_once(TimerList::new());
    });
}

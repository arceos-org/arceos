use kernel_guard::NoOp;
use lazyinit::LazyInit;
use timer_list::{TimeValue, TimerEvent, TimerList};

use axhal::time::wall_time;

use crate::{select_run_queue, AxTaskRef};

percpu_static! {
    TIMER_LIST: LazyInit<TimerList<TaskWakeupEvent>> = LazyInit::new(),
}

struct TaskWakeupEvent {
    ticket_id: u64,
    task: AxTaskRef,
}

impl TimerEvent for TaskWakeupEvent {
    fn callback(self, _now: TimeValue) {
        // Ignore the timer event if the task is not in the timer list.
        // timeout was set but not triggered (wake up by `WaitQueue::notify()`).
        // Judge if this timer event is still valid by checking the ticket ID.
        if !self.task.in_timer_list() || self.task.timer_ticket() != self.ticket_id {
            // The task is not in the timer list or the ticket ID is not matched.
            // Just ignore this timer event and return.
            return;
        }

        // Timer ticket match.
        // Mark the task as not in the timer list.
        self.task.set_in_timer_list(false);
        // Timer event is triggered, expire the ticket ID.
        self.task.timer_ticket_expire_one();
        select_run_queue::<NoOp>(self.task.clone()).unblock_task(self.task, true)
    }
}

pub fn set_alarm_wakeup(deadline: TimeValue, task: AxTaskRef) {
    TIMER_LIST.with_current(|timer_list| {
        task.set_in_timer_list(true);
        timer_list.set(
            deadline,
            TaskWakeupEvent {
                ticket_id: task.timer_ticket(),
                task,
            },
        );
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

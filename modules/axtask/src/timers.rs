use axhal::time::current_time;
use lazy_init::LazyInit;
use spinlock::SpinNoIrq;
use timer_list::{TimeValue, TimerEvent, TimerList};

use crate::{AxTaskRef, RUN_QUEUE};

// TODO: per-CPU
static TIMER_LIST: LazyInit<SpinNoIrq<TimerList<TaskWakeupEvent>>> = LazyInit::new();

struct TaskWakeupEvent(AxTaskRef);

impl TimerEvent for TaskWakeupEvent {
    fn callback(self, _now: TimeValue) {
        RUN_QUEUE.lock().unblock_task(self.0, true);
    }
}

pub fn set_wakeup(deadline: TimeValue, task: AxTaskRef) {
    TIMER_LIST.lock().set(deadline, TaskWakeupEvent(task));
}

pub fn check_events() {
    while TIMER_LIST.lock().expire_one(current_time()).is_some() {}
}

pub fn init() {
    TIMER_LIST.init_by(SpinNoIrq::new(TimerList::new()));
}

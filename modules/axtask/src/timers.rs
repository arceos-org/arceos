use crate::run_queue::LOAD_BALANCE_ARR;
use alloc::sync::Arc;
use axhal::time::current_time;
use lazy_init::LazyInit;
use load_balance::BaseLoadBalance;
use spinlock::SpinNoIrq;
use timer_list::{TimeValue, TimerEvent, TimerList};

use crate::{get_current_cpu_id, AxTaskRef, RUN_QUEUE};

// TODO: per-CPU
static TIMER_LIST: LazyInit<SpinNoIrq<TimerList<TaskWakeupEvent>>> = LazyInit::new();

struct TaskWakeupEvent(AxTaskRef);

impl TimerEvent for TaskWakeupEvent {
    fn callback(self, _now: TimeValue) {
        self.0.set_in_timer_list(false);
        let target_cpu = LOAD_BALANCE_ARR[get_current_cpu_id()].find_target_cpu(self.0.get_affinity());
        //self.0.set_queue_id(target_cpu);
        RUN_QUEUE[target_cpu].unblock_task(self.0, true);
    }
}

pub fn set_alarm_wakeup(deadline: TimeValue, task: AxTaskRef) {
    let mut timers = TIMER_LIST.lock();
    task.set_in_timer_list(true);
    timers.set(deadline, TaskWakeupEvent(task));
}

pub fn cancel_alarm(task: &AxTaskRef) {
    let mut timers = TIMER_LIST.lock();
    task.set_in_timer_list(false);
    timers.cancel(|t| Arc::ptr_eq(&t.0, task));
}

pub fn check_events() {
    loop {
        let now = current_time();
        let event = TIMER_LIST.lock().expire_one(now);
        if let Some((_deadline, event)) = event {
            event.callback(now);
        } else {
            break;
        }
    }
}

pub fn init() {
    TIMER_LIST.init_by(SpinNoIrq::new(TimerList::new()));
}

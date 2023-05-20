use alloc::sync::Arc;
use axhal::time::current_time;
use lazy_init::LazyInit;
use spinlock::SpinNoIrq;
use timer_list::{TimeValue, TimerEvent, TimerList};
use crate::run_queue::LOAD_BALANCE_ARR;
use load_balance::BaseLoadBalance;

use crate::{AxTaskRef, RUN_QUEUE, get_current_cpu_id};

// TODO: per-CPU
static TIMER_LIST: LazyInit<SpinNoIrq<TimerList<TaskWakeupEvent>>> = LazyInit::new();

struct TaskWakeupEvent(AxTaskRef);

impl TimerEvent for TaskWakeupEvent {
    fn callback(self, _now: TimeValue) {
        info!("qwq {}", get_current_cpu_id());
        info!("exit 233");
        let mut rq = RUN_QUEUE[LOAD_BALANCE_ARR[get_current_cpu_id()].find_target_cpu()].lock();
        info!("exit 234");
        self.0.set_in_timer_list(false);
        rq.unblock_task(self.0, true);
        info!("exit 1");
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
            info!("exit 5");
        } else {
            break;
        }
    }
}

pub fn init() {
    TIMER_LIST.init_by(SpinNoIrq::new(TimerList::new()));
}

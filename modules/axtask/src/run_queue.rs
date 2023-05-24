use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_init::LazyInit;
use scheduler::BaseScheduler;
use load_balance::BaseLoadBalance;
use spinlock::SpinNoIrq;

use crate::task::{CurrentTask, TaskState};
use crate::{AxTaskRef, LoadBalance, Scheduler, TaskInner, WaitQueue};

use core::sync::atomic::{AtomicIsize, Ordering};

use array_init::array_init;
use alloc::vec::Vec;

lazy_static::lazy_static! {
    pub(crate) static ref RUN_QUEUE: [LazyInit<Arc<AxRunQueue>>; axconfig::SMP] =
        array_init(|_| LazyInit::new());
    pub(crate) static ref LOAD_BALANCE_ARR: [LazyInit<Arc<LoadBalance>>; axconfig::SMP] =
        array_init(|_| LazyInit::new());
}

// TODO: per-CPU
static EXITED_TASKS: SpinNoIrq<VecDeque<AxTaskRef>> = SpinNoIrq::new(VecDeque::new());

static WAIT_FOR_EXIT: WaitQueue = WaitQueue::new();

#[percpu::def_percpu]
static IDLE_TASK: LazyInit<AxTaskRef> = LazyInit::new();

pub(crate) struct AxRunQueue {
    scheduler: SpinNoIrq<Scheduler>,
    id: usize,
}

impl AxRunQueue {
    pub fn new(id: usize) -> Self {
        let gc_task = TaskInner::new(gc_entry, "gc", axconfig::TASK_STACK_SIZE);
        let mut scheduler = SpinNoIrq::new(Scheduler::new());

        gc_task.set_queue_id(id as isize);
        scheduler.lock().add_task(gc_task);
        Self { scheduler, id}
    }

    pub fn add_task(&self, task: AxTaskRef) {
        task.set_queue_id(self.id as isize);
        debug!("task spawn: {}", task.id_name());
        assert!(task.is_ready());
        LOAD_BALANCE_ARR[self.id].add_weight(1);
        trace!("load balance weight for id {}: {}", self.id, LOAD_BALANCE_ARR[self.id].get_weight());
        trace!("add task in queue {}, now the weight is {}", self.id, LOAD_BALANCE_ARR[self.id].get_weight());
        self.scheduler.lock().add_task(task);
    }

    #[cfg(feature = "irq")]
    pub fn scheduler_timer_tick(&self) {
        let curr = crate::current();
        if !curr.is_idle() && self.scheduler.lock().task_tick(curr.as_task_ref()) {
            #[cfg(feature = "preempt")]
            curr.set_preempt_pending(true);
        }
    }

    pub fn yield_current(&self) {
        let curr = crate::current();
        debug!("task yield: {}", curr.id_name());
        assert!(curr.is_running());
        self.resched_inner(false);
    }

    pub fn set_priority(&self, prio: isize) -> bool {
        self.scheduler.lock()
            .set_priority(crate::current().as_task_ref(), prio)
    }

    #[cfg(feature = "preempt")]
    pub fn resched(&self) {
        let curr = crate::current();
        assert!(curr.is_running());

        // When we get the mutable reference of the run queue, we must
        // have held the `SpinNoIrq` lock with both IRQs and preemption
        // disabled. So we need to set `current_disable_count` to 1 in
        // `can_preempt()` to obtain the preemption permission before
        //  locking the run queue.
        let can_preempt = curr.can_preempt(1);

        debug!(
            "current task is to be preempted: {}, allow={}",
            curr.id_name(),
            can_preempt
        );
        if can_preempt {
            self.resched_inner(true);
        } else {
            curr.set_preempt_pending(true);
        }
    }

    pub fn exit_current(&self, exit_code: i32) -> ! {
        let curr = crate::current();
        debug!("task exit: {}, exit_code={}, queue_id={}", curr.id_name(), exit_code, self.id);
        assert!(curr.is_running());
        assert!(!curr.is_idle());
        if curr.is_init() {
            EXITED_TASKS.lock().clear();
            axhal::misc::terminate();
        } else {
            curr.set_state(TaskState::Exited);
            EXITED_TASKS.lock().push_back(curr.clone());
            WAIT_FOR_EXIT.notify_one_locked(false, self, self.id);
            self.resched_inner(false);
        }
        unreachable!("task exited!");
    }

    pub fn block_current<F>(&self, wait_queue_push: F)
    where
        F: FnOnce(AxTaskRef),
    {
        info!("block_current 1");
        let curr = crate::current();
        info!("task block: {}", curr.id_name());
        info!("block_current 2");
        assert!(curr.is_running());
        assert!(!curr.is_idle());

        // we must not block current task with preemption disabled.
        #[cfg(feature = "preempt")]
        assert!(curr.can_preempt(1));

        curr.set_state(TaskState::Blocked);
        info!("block_current 3");
        wait_queue_push(curr.clone());
        info!("block_current 4");
        self.resched_inner(false);
        info!("block_current 5");
    }

    pub fn unblock_task(&self, task: AxTaskRef, resched: bool) {
        debug!("task unblock: {} at cpu {}", task.id_name(), self.id);
        if task.is_blocked() {
            debug!("123");
            task.set_state(TaskState::Ready);
            debug!("234");
            task.set_queue_id(self.id as isize);
            self.scheduler.lock().add_task(task); // TODO: priority
            LOAD_BALANCE_ARR[self.id].add_weight(1);
            trace!("load balance weight for id {}: {}", self.id, LOAD_BALANCE_ARR[self.id].get_weight());
            debug!("345");
            if resched {
                #[cfg(feature = "preempt")]
                crate::current().set_preempt_pending(true);
            }
        }
        debug!("456");
    }

    #[cfg(feature = "irq")]
    pub fn sleep_until(&self, deadline: axhal::time::TimeValue) {
        let curr = crate::current();
        debug!("task sleep: {}, deadline={:?}", curr.id_name(), deadline);
        assert!(curr.is_running());
        assert!(!curr.is_idle());

        let now = axhal::time::current_time();
        if now < deadline {
            crate::timers::set_alarm_wakeup(deadline, curr.clone());
            curr.set_state(TaskState::Blocked);
            self.resched_inner(false);
        }
    }
}

impl AxRunQueue {
    /// Common reschedule subroutine. If `preempt`, keep current task's time
    /// slice, otherwise reset it.
    fn if_empty_steal(&self) {
        if self.scheduler.lock().is_empty() {
            let mut queuelock = self.scheduler.lock();
            let id = self.id;
            let next = LOAD_BALANCE_ARR[id].find_stolen_cpu_id();
            trace!("load balance weight for id {} : {}", id, LOAD_BALANCE_ARR[id].get_weight());
            assert!(LOAD_BALANCE_ARR[id].get_weight() == 0);
            debug!("steal: current = {}, victim = {}", self.id, next);
            if next != -1 {
                debug!("steal 1");
                //info!("exit 233");
                let task = RUN_QUEUE[next as usize].scheduler.lock().pick_next_task();
                //info!("exit 234");
                debug!("steal 2");
                // 这里可能有同步问题，简单起见，如果 task 是 None 那么就不窃取。
                if let Some(tk) = task {
                    tk.set_queue_id(self.id as isize);
                    queuelock.add_task(tk);
                    debug!("steal 3");
                    LOAD_BALANCE_ARR[next as usize].add_weight(-1);
                    trace!("load balance weight for id {}: {}", next as usize, LOAD_BALANCE_ARR[next as usize].get_weight());
                    debug!("steal 4");
                    LOAD_BALANCE_ARR[id].add_weight(1);
                    trace!("load balance weight for id {}: {}", id, LOAD_BALANCE_ARR[id].get_weight());
                    debug!("steal 5");
                }
            }
        }
    }
    fn resched_inner(&self, preempt: bool) {
        debug!("resched inner 1");
        let prev = crate::current();
        debug!("resched inner 2");
        if prev.is_running() {
            debug!("resched inner 3");
            prev.set_state(TaskState::Ready);
            debug!("resched inner 4");
            if !prev.is_idle() {
                debug!("resched inner 5");
                prev.set_queue_id(self.id as isize);
                self.scheduler.lock().put_prev_task(prev.clone(), preempt);
                LOAD_BALANCE_ARR[self.id].add_weight(1); //?
                trace!("load balance weight for id {}: {}", self.id, LOAD_BALANCE_ARR[self.id].get_weight());
                debug!("resched inner 6");
            }
        }
        debug!("resched inner 7");
        let mut flag = false;
        let next = self.scheduler.lock().pick_next_task().unwrap_or_else(|| unsafe {
            // Safety: IRQs must be disabled at this time.
            LOAD_BALANCE_ARR[self.id].add_weight(1); // 后面需要减一，由于是 IDLE 所以不用减，先加一
            trace!("load balance weight for id {}: {}", self.id, LOAD_BALANCE_ARR[self.id].get_weight());
            flag = true;
            IDLE_TASK.current_ref_raw().get_unchecked().clone()
        });
        if !flag {
            next.set_queue_id(-1);
        }
        LOAD_BALANCE_ARR[self.id].add_weight(-1); //?
        trace!("load balance weight for id {}: {}", self.id, LOAD_BALANCE_ARR[self.id].get_weight());
        debug!("resched inner 8");
        // TODO: 注意需要对所有 pick_next_task 后面都要判断是否队列空，如果是则需要执行线程窃取
        self.if_empty_steal();
        self.switch_to(prev, next);
        debug!("resched inner 9");
    }

    fn switch_to(&self, prev_task: CurrentTask, next_task: AxTaskRef) {
        trace!(
            "context switch: {} -> {}",
            prev_task.id_name(),
            next_task.id_name()
        );
        #[cfg(feature = "preempt")]
        next_task.set_preempt_pending(false);
        next_task.set_state(TaskState::Running);
        if prev_task.ptr_eq(&next_task) {
            return;
        }

        unsafe {
            let prev_ctx_ptr = prev_task.ctx_mut_ptr();
            let next_ctx_ptr = next_task.ctx_mut_ptr();

            // The strong reference count of `prev_task` will be decremented by 1,
            // but won't be dropped until `gc_entry()` is called.
            assert!(Arc::strong_count(prev_task.as_task_ref()) > 1);
            assert!(Arc::strong_count(&next_task) >= 1);

            CurrentTask::set_current(prev_task, next_task);
            (*prev_ctx_ptr).switch_to(&*next_ctx_ptr);
        }
    }
}

fn gc_entry() {
    loop {
        debug!("gc1");
        // Drop all exited tasks and recycle resources.
        while !EXITED_TASKS.lock().is_empty() {
            debug!("gc2");
            // Do not do the slow drops in the critical section.
            let task = EXITED_TASKS.lock().pop_front();
            debug!("gc3");
            if let Some(task) = task {
                // wait for other threads to release the reference.
        debug!("gc4");
                while Arc::strong_count(&task) > 1 {
                    core::hint::spin_loop();
                }
                debug!("gc5");
                drop(task);
            }
            debug!("gc6");
        }
        debug!("gc7");
        WAIT_FOR_EXIT.wait();
        debug!("gc8");
    }
}

pub(crate) fn init() {
    const IDLE_TASK_STACK_SIZE: usize = 4096;
    let idle_task = TaskInner::new(|| crate::run_idle(), "idle", IDLE_TASK_STACK_SIZE);
    IDLE_TASK.with_current(|i| i.init_by(idle_task.clone()));

    let main_task = TaskInner::new_init("main");
    main_task.set_state(TaskState::Running);

    for i in 0..axconfig::SMP {
        RUN_QUEUE[i].init_by(Arc::new(AxRunQueue::new(i)));
        LOAD_BALANCE_ARR[i].init_by(Arc::new(LoadBalance::new(i)));
        LOAD_BALANCE_ARR[i].add_weight(1); // gc_task
    }
    let mut arr = Vec::new();
    for i in 0..axconfig::SMP {
        arr.push((*LOAD_BALANCE_ARR[i]).clone());
    }
    for i in 0..axconfig::SMP {
        LOAD_BALANCE_ARR[i].init(axconfig::SMP, arr.clone());
    }
    unsafe { CurrentTask::init_current(main_task) }
}

pub(crate) fn init_secondary() {
    let idle_task = TaskInner::new_init("idle");
    idle_task.set_state(TaskState::Running);
    IDLE_TASK.with_current(|i| i.init_by(idle_task.clone()));
    unsafe { CurrentTask::init_current(idle_task) }
}

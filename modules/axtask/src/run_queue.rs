use alloc::sync::Arc;
use alloc::{collections::VecDeque, vec::Vec};
use core::sync::atomic::{AtomicUsize, Ordering};

use array_init::array_init;
use lazy_init::LazyInit;
use load_balance::BaseLoadBalance;
use scheduler::BaseScheduler;
use spinlock::SpinNoIrq;

use crate::task::{CurrentTask, TaskState};
use crate::{AxTaskRef, LoadBalance, Scheduler, TaskInner, WaitQueue};

lazy_static::lazy_static! {
    pub(crate) static ref RUN_QUEUE: [LazyInit<Arc<AxRunQueue>>; axconfig::SMP] =
        array_init(|_| LazyInit::new());
    pub(crate) static ref LOAD_BALANCE_ARR: [LazyInit<Arc<LoadBalance>>; axconfig::SMP] =
        array_init(|_| LazyInit::new());
}

// TODO: per-CPU
static EXITED_TASKS: SpinNoIrq<VecDeque<AxTaskRef>> = SpinNoIrq::new(VecDeque::new());

static WAIT_FOR_EXIT: WaitQueue = WaitQueue::new();

static SWITCH_EXITED_LOCK: AtomicUsize = AtomicUsize::new(0);

use kernel_guard::NoPreempt;

#[percpu::def_percpu]
static IDLE_TASK: LazyInit<AxTaskRef> = LazyInit::new();

pub(crate) struct AxRunQueue {
    scheduler: SpinNoIrq<Scheduler>,
    id: usize,
}

impl AxRunQueue {
    pub fn new(id: usize) -> Self {
        let gc_task = TaskInner::new(gc_entry, "gc".into(), axconfig::TASK_STACK_SIZE);
        let scheduler = SpinNoIrq::new(Scheduler::new());

        gc_task.set_queue_id(id as isize);
        // this task must run on gc_task
        gc_task.set_affinity(1u64 << id);
        scheduler.lock().add_task(gc_task);
        Self { scheduler, id }
    }

    pub fn add_task(&self, task: AxTaskRef) {
        task.set_queue_id(self.id as isize);
        debug!(
            "task spawn: {} at queue {}, affinity is {}",
            task.id_name(),
            self.id,
            task.get_affinity()
        );
        assert!(task.is_ready());
        LOAD_BALANCE_ARR[self.id].add_weight(1);
        trace!(
            "load balance weight for id {}: {}",
            self.id,
            LOAD_BALANCE_ARR[self.id].get_weight()
        );
        trace!(
            "add task in queue {}, now the weight is {}",
            self.id,
            LOAD_BALANCE_ARR[self.id].get_weight()
        );
        self.scheduler.lock().add_task(task);
    }

    #[cfg(feature = "irq")]
    pub fn scheduler_timer_tick(&self) {
        //let tmp = LOCK_QWQ3.lock();
        let curr = crate::current();
        //info!("qwq1");
        if !curr.is_idle() && self.scheduler.lock().task_tick(curr.as_task_ref()) {
            #[cfg(feature = "preempt")]
            curr.set_preempt_pending(true);
        }
    }

    pub fn yield_current(&self) {
        let curr = crate::current();
        debug!("task yield: {}", curr.id_name());
        assert!(curr.is_running());
        self.resched_inner(false, false);
    }

    pub fn set_current_priority(&self, prio: isize) -> bool {
        self.scheduler
            .lock()
            .set_priority(crate::current().as_task_ref(), prio)
    }

    #[cfg(feature = "preempt")]
    pub fn resched(&self) {
        //info!("resched");
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
            self.resched_inner(true, false);
        } else {
            curr.set_preempt_pending(true);
        }
    }

    pub fn exit_current(&self, exit_code: i32) -> ! {
        let curr = crate::current();
        debug!(
            "task exit: {}, exit_code={}, queue_id={}",
            curr.id_name(),
            exit_code,
            self.id
        );
        assert!(curr.is_running());
        assert!(!curr.is_idle());
        if curr.is_init() {
            EXITED_TASKS.lock().clear();
            axhal::misc::terminate();
        } else {
            curr.set_state(TaskState::Exited);
            curr.notify_exit(exit_code, self);
            //SWITCH_EXITED_LOCK.fetch_add(1, Ordering::Release);
            EXITED_TASKS.lock().push_back(curr.clone());
            WAIT_FOR_EXIT.notify_one_locked(false);
            self.resched_inner(false, true);
        }
        unreachable!("task exited!");
    }

    pub fn block_current<F>(&self, wait_queue_push: F)
    where
        F: FnOnce(AxTaskRef),
    {
        let curr = crate::current();
        debug!("task block: {}", curr.id_name());
        assert!(curr.is_running());
        assert!(!curr.is_idle());

        // we must not block current task with preemption disabled.
        #[cfg(feature = "preempt")]
        assert!(curr.can_preempt(1));

        curr.set_state(TaskState::Blocked);
        wait_queue_push(curr.clone());
        self.resched_inner(false, false);
    }

    pub fn unblock_task(&self, task: AxTaskRef, resched: bool) {
        debug!("task unblock: {} at cpu {}", task.id_name(), self.id);
        if task.is_blocked() {
            task.set_state(TaskState::Ready);
            task.set_queue_id(self.id as isize);
            self.scheduler.lock().add_task(task); // TODO: priority
            LOAD_BALANCE_ARR[self.id].add_weight(1);
            trace!(
                "load balance weight for id {}: {}",
                self.id,
                LOAD_BALANCE_ARR[self.id].get_weight()
            );
            if resched {
                #[cfg(feature = "preempt")]
                crate::current().set_preempt_pending(true);
            }
        }
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
            self.resched_inner(false, false);
        }
    }
}

impl AxRunQueue {
    
    pub fn with_current_rq<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&AxRunQueue) -> T,
    {
        let _guard = kernel_guard::NoPreempt::new();
        f(&RUN_QUEUE[axhal::cpu::this_cpu_id()])
    }
    
    pub fn with_task_correspond_rq<F, T>(&self, task: AxTaskRef, f: F) -> T
    where
        F: FnOnce(&AxRunQueue) -> T,
    {
        let _guard = kernel_guard::NoPreempt::new();
        let target = LOAD_BALANCE_ARR[axhal::cpu::this_cpu_id()].find_target_cpu(task.get_affinity());
        f(&RUN_QUEUE[target])
    }
    /// Common reschedule subroutine. If `preempt`, keep current task's time
    /// slice, otherwise reset it.
    fn if_empty_steal(&self) {
        return;
        if self.scheduler.lock().is_empty() {
            let mut forbidden: u64 = 0;
            loop {
                let mut queuelock = self.scheduler.lock();
                let id = self.id;
                let next = LOAD_BALANCE_ARR[id].find_stolen_cpu_id();
                trace!(
                    "load balance weight for id {} : {}",
                    id,
                    LOAD_BALANCE_ARR[id].get_weight()
                );
                debug!("steal: current = {}, victim = {}", self.id, next);
                if next != -1 {
                    let task = RUN_QUEUE[next as usize].scheduler.lock().pick_next_task();
                    // 这里可能有同步问题，简单起见，如果 task 是 None 那么就不窃取。
                    if let Some(tk) = task {
                        assert!(tk.get_queue_id() == next);
                        tk.set_queue_id(id as isize);
                        queuelock.add_task(tk);
                        LOAD_BALANCE_ARR[next as usize].add_weight(-1);
                        trace!(
                            "load balance weight for id {}: {}",
                            next as usize,
                            LOAD_BALANCE_ARR[next as usize].get_weight()
                        );
                        LOAD_BALANCE_ARR[id].add_weight(1);
                        trace!(
                            "load balance weight for id {}: {}",
                            id,
                            LOAD_BALANCE_ARR[id].get_weight()
                        );
                    }
                }
            }
        }
    }
    fn resched_inner(&self, preempt: bool, exit_lock: bool) {
        let prev = crate::current();
        if prev.is_running() {
            prev.set_state(TaskState::Ready);
            if !prev.is_idle() {
                prev.set_queue_id(self.id as isize);
                self.scheduler.lock().put_prev_task(prev.clone(), preempt);
                LOAD_BALANCE_ARR[self.id].add_weight(1);
                trace!(
                    "load balance weight for id {}: {}",
                    self.id,
                    LOAD_BALANCE_ARR[self.id].get_weight()
                );
            }
        }
        let mut flag = false;
        let next = self
            .scheduler
            .lock()
            .pick_next_task()
            .unwrap_or_else(|| unsafe {
                // Safety: IRQs must be disabled at this time.
                trace!(
                    "load balance weight for id {}: {}",
                    self.id,
                    LOAD_BALANCE_ARR[self.id].get_weight()
                );
                LOAD_BALANCE_ARR[self.id].add_weight(1); // 后面需要减一，由于是 IDLE 所以不用减，先加一
                flag = true;
                IDLE_TASK.current_ref_raw().get_unchecked().clone()
            });
        /*if !flag {
            next.set_queue_id(-1);
        }*/
        LOAD_BALANCE_ARR[self.id].add_weight(-1);
        trace!(
            "load balance weight for id {}: {}",
            self.id,
            LOAD_BALANCE_ARR[self.id].get_weight()
        );
        // TODO: 注意需要对所有 pick_next_task 后面都要判断是否队列空，如果是则需要执行线程窃取
        self.if_empty_steal();
        self.switch_to(prev, next, exit_lock);
    }

    fn switch_to(&self, prev_task: CurrentTask, next_task: AxTaskRef, exit_lock: bool) {
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
            /*if exit_lock {
                SWITCH_EXITED_LOCK.store(
                    SWITCH_EXITED_LOCK.load(Ordering::Acquire) - 1,
                    Ordering::Release,
                );
            }*/
            (*prev_ctx_ptr).switch_to(&*next_ctx_ptr);
        }
    }
}

fn gc_entry() {
    loop {
        // Drop all exited tasks and recycle resources.
        while !EXITED_TASKS.lock().is_empty() {
            // 用 lock 先顶一顶
           // while SWITCH_EXITED_LOCK.load(Ordering::Acquire) > 0 {
            //    trace!("qwqq {}", SWITCH_EXITED_LOCK.load(Ordering::Acquire));
            //}
            // Do not do the slow drops in the critical section.
            let task = EXITED_TASKS.lock().pop_front();
            if let Some(task) = task {
                // If the task reference is not taken after `spawn()`, it will be
                // dropped here. Otherwise, it will be dropped after the reference
                // is dropped (usually by `join()`).
                if Arc::strong_count(&task) == 1 {
                    drop(task);
                } else {
                    EXITED_TASKS.lock().push_back(task);
                    // TODO: for loop
                    break;
                }
            }
        }
        WAIT_FOR_EXIT.wait();
    }
}

pub(crate) fn init() {
    const IDLE_TASK_STACK_SIZE: usize = 4096;
    let idle_task = TaskInner::new(|| crate::run_idle(), "idle".into(), IDLE_TASK_STACK_SIZE);
    IDLE_TASK.with_current(|i| i.init_by(idle_task.clone()));

    let main_task = TaskInner::new_init("main".into());
    main_task.set_state(TaskState::Running);
    main_task.set_affinity((1u64 << axconfig::SMP) - 1);

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
    let idle_task = TaskInner::new_init("idle".into());
    idle_task.set_state(TaskState::Running);
    IDLE_TASK.with_current(|i| i.init_by(idle_task.clone()));
    unsafe { CurrentTask::init_current(idle_task) }
}

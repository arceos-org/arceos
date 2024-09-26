# About How to support percpu scheduler in ArceOS.

## Background

Orginally, ArceOS uses a rude global RunQueue, and scheduling operations like 
task yielding, waiting on wait queue and notifying a blocked task are 
all under the protection of  a global SpinLockNoIrq hold by RunQueue.

To support percpu scheduling, we must refactor the run queue structure,  
as well as the locking mechanism in the current scheduling framework.

## AxRunQueue and Scheduler crate

For the design of the scheduler interface, we can reference the design used in Linux:

```C
// kernel/sched/sched.h

struct sched_class {

#ifdef CONFIG_UCLAMP_TASK
	int uclamp_enabled;
#endif

	void (*enqueue_task) (struct rq *rq, struct task_struct *p, int flags);
	void (*dequeue_task) (struct rq *rq, struct task_struct *p, int flags);
	void (*yield_task)   (struct rq *rq);
	bool (*yield_to_task)(struct rq *rq, struct task_struct *p);

	void (*wakeup_preempt)(struct rq *rq, struct task_struct *p, int flags);

	struct task_struct *(*pick_next_task)(struct rq *rq);

	void (*put_prev_task)(struct rq *rq, struct task_struct *p);
	void (*set_next_task)(struct rq *rq, struct task_struct *p, bool first);

#ifdef CONFIG_SMP
	int (*balance)(struct rq *rq, struct task_struct *prev, struct rq_flags *rf);
	int  (*select_task_rq)(struct task_struct *p, int task_cpu, int flags);

	struct task_struct * (*pick_task)(struct rq *rq);

	void (*migrate_task_rq)(struct task_struct *p, int new_cpu);

	void (*task_woken)(struct rq *this_rq, struct task_struct *task);

	void (*set_cpus_allowed)(struct task_struct *p, struct affinity_context *ctx);

	void (*rq_online)(struct rq *rq);
	void (*rq_offline)(struct rq *rq);

	struct rq *(*find_lock_rq)(struct task_struct *p, struct rq *rq);
#endif

	void (*task_tick)(struct rq *rq, struct task_struct *p, int queued);
	void (*task_fork)(struct task_struct *p);
	void (*task_dead)(struct task_struct *p);

	/*
	 * The switched_from() call is allowed to drop rq->lock, therefore we
	 * cannot assume the switched_from/switched_to pair is serialized by
	 * rq->lock. They are however serialized by p->pi_lock.
	 */
	void (*switched_from)(struct rq *this_rq, struct task_struct *task);
	void (*switched_to)  (struct rq *this_rq, struct task_struct *task);
	void (*prio_changed) (struct rq *this_rq, struct task_struct *task,
			      int oldprio);

	unsigned int (*get_rr_interval)(struct rq *rq,
					struct task_struct *task);

	void (*update_curr)(struct rq *rq);

#ifdef CONFIG_FAIR_GROUP_SCHED
	void (*task_change_group)(struct task_struct *p);
#endif

#ifdef CONFIG_SCHED_CORE
	int (*task_is_throttled)(struct task_struct *p, int cpu);
#endif
};
```

Current [`scheduler`](https://github.com/arceos-org/scheduler) crate used by ArceOS 
provides a more fundamental scheduling method interface, which only includes 
basic task operations and does not account for multiple run queues:

```Rust
/// The base scheduler trait that all schedulers should implement.
///
/// All tasks in the scheduler are considered runnable. If a task is go to
/// sleep, it should be removed from the scheduler.
pub trait BaseScheduler {
    /// Type of scheduled entities. Often a task struct.
    type SchedItem;

    /// Initializes the scheduler.
    fn init(&mut self);

    /// Adds a task to the scheduler.
    fn add_task(&mut self, task: Self::SchedItem);

    /// Removes a task by its reference from the scheduler. Returns the owned
    /// removed task with ownership if it exists.
    ///
    /// # Safety
    ///
    /// The caller should ensure that the task is in the scheduler, otherwise
    /// the behavior is undefined.
    fn remove_task(&mut self, task: &Self::SchedItem) -> Option<Self::SchedItem>;

    /// Picks the next task to run, it will be removed from the scheduler.
    /// Returns [`None`] if there is not runnable task.
    fn pick_next_task(&mut self) -> Option<Self::SchedItem>;

    /// Puts the previous task back to the scheduler. The previous task is
    /// usually placed at the end of the ready queue, making it less likely
    /// to be re-scheduled.
    ///
    /// `preempt` indicates whether the previous task is preempted by the next
    /// task. In this case, the previous task may be placed at the front of the
    /// ready queue.
    fn put_prev_task(&mut self, prev: Self::SchedItem, preempt: bool);

    /// Advances the scheduler state at each timer tick. Returns `true` if
    /// re-scheduling is required.
    ///
    /// `current` is the current running task.
    fn task_tick(&mut self, current: &Self::SchedItem) -> bool;

    /// Set priority for a task.
    fn set_priority(&mut self, task: &Self::SchedItem, prio: isize) -> bool;
}
```

The current scheduler design focuses solely on the task states within its own ready queue. 
The scheduler is held by AxRunQueue as a global static variable 
and serves as a globally unique scheduler for all cores.

```Rust
// modules/axtask/src/run_queue.rs

// TODO: per-CPU
pub(crate) static RUN_QUEUE: LazyInit<SpinNoIrq<AxRunQueue>> = LazyInit::new();

pub(crate) struct AxRunQueue {
    scheduler: Scheduler,
}
```

Referencing Linux's design, methods such as `select_task_rq` and those related to 
load balancing should be provided by the scheduler itself. 
However, to simplify the design and minimize modifications to the scheduler crate, 
we continue to use ArceOS's original design, managing the scheduler with AxRunQueue. 
We will change `AxRunQueue` to be a per-CPU variable instead of a globally unique instance 
(as well as `EXITED_TASKS`, `WAIT_FOR_EXIT`, and `TIMER_LIST`).

By doing this, the originally global unique SpinNoIrq of AxRunQueue needs to be distributed across each core. 
We will refactor the locking mechanism and refine the granularity of the locks.

## cpumask crate

We introduce [cpumask](https://github.com/arceos-org/cpumask) crate for CPU affinity attribute for a task.

## Lock, Irq and Preemption

### AxRunQueue
The lock for AxRunQueue no longer exists. 

For the run queue, we have refined the locks to the ready queue within the scheduler, 
meaning that only operations that require interaction with the ready queue, 
such as picking the next task and pushing tasks, need to be locked.

The current run queue for a core can be obtained through the `current_run_queue` method. 
This process needs to be performed under the protection of `kernel_guard` to ensure the safety of preemption and interrupt states.

```Rust
/// Returns a reference to the current run queue.
///
/// ## Safety
///
/// This function returns a static reference to the current run queue, which
/// is inherently unsafe. It assumes that the `RUN_QUEUE` has been properly
/// initialized and is not accessed concurrently in a way that could cause
/// data races or undefined behavior.
///
/// ## Returns
///
/// A static reference to the current run queue.
// #[inline(always)]
pub(crate) fn current_run_queue<G: BaseGuard>() -> AxRunQueueRef<'static, G> {
    let irq_state = G::acquire();
    AxRunQueueRef {
        inner: unsafe { RUN_QUEUE.current_ref_mut_raw() },
        state: irq_state,
        _phantom: core::marker::PhantomData,
    }
}
```

### WaitQueue
Operations on the wait queue are no longer protected by the large lock of AxRunQueue. 

We need to protect the wait queue using `SpinNoIrq` and distinguish it from operations related to the run queue:
* When waiting for a task, first insert it into the wait queue, then call the relevant methods of the run queue for task switching.
* When waking up a task, first remove it from the wait queue, then call the `select_run_queue` method to choose an appropriate run queue for insertion.

### TimerList

The TimerList is also designed to be per-CPU, used for recording and responding to specific clock times. 
This allows us to eliminate the lock for TimerList itself. 

TimerList may be used in `wait_timeout_until`, where a task can simultaneously wait for either a notification or a timer event. 
Therefore, a task may be placed in both TimerList and WaitQueue. 

To prevent a task from being awakened by both methods simultaneously, we need to apply an `unblock_lock` to the task, ensuring that the unblock operation for a task can **succeed only once**.

```Rust
    pub(crate) fn unblock_locked<F>(&self, mut run_queue_push: F)
    where
        F: FnMut(),
    {
        debug!("{} unblocking", self.id_name());

        // If the owning (remote) CPU is still in the middle of schedule() with
        // this task as prev, wait until it's done referencing the task.
        //
        // Pairs with the `clear_prev_task_on_cpu()`.
        //
        // This ensures that tasks getting woken will be fully ordered against
        // their previous state and preserve Program Order.
        if self.on_cpu() {
            while self.on_cpu() {
                // Wait for the task to finish its scheduling process.
                core::hint::spin_loop();
            }
            assert!(!self.on_cpu())
        }

        // When irq is enabled, use `unblock_lock` to protect the task from being unblocked by timer and `notify()` at the same time.
        #[cfg(feature = "irq")]
        let _lock = self.unblock_lock.lock();
        if self.is_blocked() {
            run_queue_push();
        }
    }
```

## The `on_cpu` flag


When we reduce the lock granularity of the run queue and distinguish it from the wait queue locks, 
we need to address a phenomenon: 
when a task calls a `wait_xxx` method to wait on a specific wait queue, 
it may not have been scheduled away from its current CPU before being woken up by another CPU's wait queue and running on that CPU. 
The general flow may be as follows:

```
CPU 0               |   CPU1
wq.lock()               
push A to wq
wq.unlock()             
                        wq.lock()
                        pop A from wq
                        wq.unlock()
...                     ...
...                     save prev_ctx
-------------------------------------------
save prev_ctx(A)        restore next_ctx(A)
-------------------------------------------
restore next_ctx
```

We have to use some stragety to ensure read-after-write consistency.

* shenango and skyloft introduce a `stack_busy` flag in task struct to indicate whether the task has finishes switching stacks,
it is set as true for a task when yielding is about to happened, and cleared after its context has been saved to stack.

    ```ASM
    .align 16
    .globl __context_switch
    .type __context_switch, @function
    __context_switch:
        SAVE_CALLEE
        SAVE_FXSTATE

        mov [rdi], rsp

        /* clear the stack busy flag */
        mov byte ptr [rdx], 0

        mov rsp, rsi

        RESTORE_FXSTATE
        RESTORE_CALLEE
    #ifdef SKYLOFT_UINTR
        /* enable preemption */
        stui
    #endif
        ret
    ```

    During scheduling process, when it tries to yield to a task with `stack_busy` true, it need to enter a spin loop:

    ```C
        /* task must be scheduled atomically */
        if (unlikely(atomic_load_acq(&next->stack_busy))) {
            /* wait until the scheduler finishes switching stacks */
            while (atomic_load_acq(&next->stack_busy)) cpu_relax();
        }
    ```

* Linux use a `on_cpu` flag 

    ```C
    * p->on_cpu <- { 0, 1 }:
    *
    *   is set by prepare_task() and cleared by finish_task() such that it will be
    *   set before p is scheduled-in and cleared after p is scheduled-out, both
    *   under rq->lock. Non-zero indicates the task is running on its CPU.
    *
    *   [ The astute reader will observe that it is possible for two tasks on one
    *     CPU to have ->on_cpu = 1 at the same time. ]
    ```


    During a scheduling event in Linux, the process begins by calling `prepare_task` to set the `on_cpu` flag of the next task to 1. 
    After invoking the `switch_to` method to switch to the next task, 
    the next task receives a return value pointing to the previous task's pointer, 
    allowing it to clear the `on_cpu` flag of the previous task.
    Basic workflow:
    ```C
    // on prev task
    context_switch
        prepare_task_switch(rq, prev, next);
            prepare_task(next);
                WRITE_ONCE(next->on_cpu, 1);
        switch_to(prev, next, prev);
            ((last) = __switch_to_asm((prev), (next)));
    // On next task
        finish_task_switch(prev);   
            finish_task(prev);
                smp_store_release(&prev->on_cpu, 0);
    ```
    The TTWU_QUEUE feature in Linux allows the use of IPI to wake up a remote CPU within the `try_to_wake_up` function, 
    instead of waiting for the task on the remote CPU to complete its scheduling process. 
    This reduces the overhead of spinlocks and locks.

    ```C
    // kernel/sched/core.c

    int try_to_wake_up() {
        // ...

        /*
        * If the owning (remote) CPU is still in the middle of schedule() with
        * this task as prev, considering queueing p on the remote CPUs wake_list
        * which potentially sends an IPI instead of spinning on p->on_cpu to
        * let the waker make forward progress. This is safe because IRQs are
        * disabled and the IPI will deliver after on_cpu is cleared.
        *
        * Ensure we load task_cpu(p) after p->on_cpu:
        *
        * set_task_cpu(p, cpu);
        *   STORE p->cpu = @cpu
        * __schedule() (switch to task 'p')
        *   LOCK rq->lock
        *   smp_mb__after_spin_lock()		smp_cond_load_acquire(&p->on_cpu)
        *   STORE p->on_cpu = 1		LOAD p->cpu
        *
        * to ensure we observe the correct CPU on which the task is currently
        * scheduling.
        */
        if (smp_load_acquire(&p->on_cpu) &&
            ttwu_queue_wakelist(p, task_cpu(p), wake_flags))
            break;

        /*
        * If the owning (remote) CPU is still in the middle of schedule() with
        * this task as prev, wait until it's done referencing the task.
        *
        * Pairs with the smp_store_release() in finish_task().
        *
        * This ensures that tasks getting woken will be fully ordered against
        * their previous state and preserve Program Order.
        */
        smp_cond_load_acquire(&p->on_cpu, !VAL);
    }
    ```

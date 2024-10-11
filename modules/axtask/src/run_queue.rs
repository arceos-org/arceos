use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::mem::MaybeUninit;

use kernel_guard::BaseGuard;
use kspin::SpinRaw;
use lazyinit::LazyInit;
use scheduler::BaseScheduler;

use axhal::cpu::this_cpu_id;

use crate::task::{CurrentTask, TaskState};
use crate::wait_queue::WaitQueueGuard;
use crate::{AxTaskRef, CpuMask, Scheduler, TaskInner, WaitQueue};

macro_rules! percpu_static {
    ($($name:ident: $ty:ty = $init:expr),* $(,)?) => {
        $(
            #[percpu::def_percpu]
            static $name: $ty = $init;
        )*
    };
}

percpu_static! {
    RUN_QUEUE: LazyInit<AxRunQueue> = LazyInit::new(),
    EXITED_TASKS: VecDeque<AxTaskRef> = VecDeque::new(),
    WAIT_FOR_EXIT: WaitQueue = WaitQueue::new(),
    IDLE_TASK: LazyInit<AxTaskRef> = LazyInit::new(),
}

/// An array of references to run queues, one for each CPU, indexed by cpu_id.
///
/// This static variable holds references to the run queues for each CPU in the system.
///
/// # Safety
///
/// Access to this variable is marked as `unsafe` because it contains `MaybeUninit` references,
/// which require careful handling to avoid undefined behavior. The array should be fully
/// initialized before being accessed to ensure safe usage.
static mut RUN_QUEUES: [MaybeUninit<&'static mut AxRunQueue>; axconfig::SMP] =
    [ARRAY_REPEAT_VALUE; axconfig::SMP];
const ARRAY_REPEAT_VALUE: MaybeUninit<&'static mut AxRunQueue> = MaybeUninit::uninit();

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
#[inline(always)]
pub(crate) fn current_run_queue<G: BaseGuard>() -> AxRunQueueRef<'static, G> {
    let irq_state = G::acquire();
    AxRunQueueRef {
        inner: unsafe { RUN_QUEUE.current_ref_mut_raw() },
        state: irq_state,
        _phantom: core::marker::PhantomData,
    }
}

/// Selects the run queue index based on a CPU set bitmap and load balancing.
///
/// This function filters the available run queues based on the provided `cpumask` and
/// selects the run queue index for the next task. The selection is based on a round-robin algorithm.
///
/// ## Arguments
///
/// * `cpumask` - A bitmap representing the CPUs that are eligible for task execution.
///
/// ## Returns
///
/// The index (cpu_id) of the selected run queue.
///
/// ## Panics
///
/// This function will panic if `cpu_mask` is empty, indicating that there are no available CPUs for task execution.
///
#[cfg(feature = "smp")]
// The modulo operation is safe here because `axconfig::SMP` is always greater than 1 with "smp" enabled.
#[allow(clippy::modulo_one)]
#[inline]
fn select_run_queue_index(cpumask: CpuMask) -> usize {
    use core::sync::atomic::{AtomicUsize, Ordering};
    static RUN_QUEUE_INDEX: AtomicUsize = AtomicUsize::new(0);

    assert!(!cpumask.is_empty(), "No available CPU for task execution");

    // Round-robin selection of the run queue index.
    loop {
        let index = RUN_QUEUE_INDEX.fetch_add(1, Ordering::SeqCst) % axconfig::SMP;
        if cpumask.get(index) {
            return index;
        }
    }
}

/// Retrieves a `'static` reference to the run queue corresponding to the given index.
///
/// This function asserts that the provided index is within the range of available CPUs
/// and returns a reference to the corresponding run queue.
///
/// ## Arguments
///
/// * `index` - The index of the run queue to retrieve.
///
/// ## Returns
///
/// A reference to the `AxRunQueue` corresponding to the provided index.
///
/// ## Panics
///
/// This function will panic if the index is out of bounds.
///
#[cfg(feature = "smp")]
#[inline]
fn get_run_queue(index: usize) -> &'static mut AxRunQueue {
    unsafe { RUN_QUEUES[index].assume_init_mut() }
}

/// Selects the appropriate run queue for the provided task.
///
/// * In a single-core system, this function always returns a reference to the global run queue.
/// * In a multi-core system, this function selects the run queue based on the task's CPU affinity and load balance.
///
/// ## Arguments
///
/// * `task` - A reference to the task for which a run queue is being selected.
///
/// ## Returns
///
/// A reference to the selected `AxRunQueue`.
///
/// ## TODO
///
/// 1. Implement better load balancing across CPUs for more efficient task distribution.
/// 2. Use a more generic load balancing algorithm that can be customized or replaced.
///
#[inline]
pub(crate) fn select_run_queue<G: BaseGuard>(task: &AxTaskRef) -> AxRunQueueRef<'static, G> {
    #[cfg(not(feature = "smp"))]
    {
        let _ = task;
        // When SMP is disabled, all tasks are scheduled on the same global run queue.
        current_run_queue()
    }
    #[cfg(feature = "smp")]
    {
        let irq_state = G::acquire();
        // When SMP is enabled, select the run queue based on the task's CPU affinity and load balance.
        let index = select_run_queue_index(task.cpumask());
        AxRunQueueRef {
            inner: get_run_queue(index),
            state: irq_state,
            _phantom: core::marker::PhantomData,
        }
    }
}

/// `AxRunQueue` represents a run queue for global system or a specific CPU.
pub(crate) struct AxRunQueue {
    /// The ID of the CPU this run queue is associated with.
    cpu_id: usize,
    /// The core scheduler of this run queue.
    /// Since irq and preempt are preserved by the kernel guard hold by `AxRunQueueRef`,
    /// we just use a simple raw spin lock here.
    scheduler: SpinRaw<Scheduler>,
}

pub(crate) struct AxRunQueueRef<'a, G: BaseGuard> {
    inner: &'a mut AxRunQueue,
    state: G::State,
    _phantom: core::marker::PhantomData<G>,
}

impl<'a, G: BaseGuard> Drop for AxRunQueueRef<'a, G> {
    fn drop(&mut self) {
        G::release(self.state);
    }
}

/// Core functions of run queue.
impl<'a, G: BaseGuard> AxRunQueueRef<'a, G> {
    pub fn add_task(&mut self, task: AxTaskRef) {
        debug!(
            "task add: {} on run_queue {}",
            task.id_name(),
            self.inner.cpu_id
        );
        assert!(task.is_ready());
        self.inner.scheduler.lock().add_task(task);
    }

    #[cfg(feature = "irq")]
    pub fn scheduler_timer_tick(&mut self) {
        let curr = crate::current();
        if !curr.is_idle() && self.inner.scheduler.lock().task_tick(curr.as_task_ref()) {
            #[cfg(feature = "preempt")]
            curr.set_preempt_pending(true);
        }
    }

    pub fn yield_current(&mut self) {
        let curr = crate::current();
        trace!("task yield: {}", curr.id_name());
        assert!(curr.is_running());
        self.inner.resched(false);
    }

    pub fn set_current_priority(&mut self, prio: isize) -> bool {
        self.inner
            .scheduler
            .lock()
            .set_priority(crate::current().as_task_ref(), prio)
    }

    #[cfg(feature = "preempt")]
    pub fn preempt_resched(&mut self) {
        // There is no need to disable IRQ and preemption here, because
        // they both have been disabled in `current_check_preempt_pending`.
        let curr = crate::current();
        assert!(curr.is_running());

        // When we call `preempt_resched()`, both IRQs and preemption must
        // have been disabled by `kernel_guard::NoPreemptIrqSave`. So we need
        // to set `current_disable_count` to 1 in `can_preempt()` to obtain
        // the preemption permission.
        let can_preempt = curr.can_preempt(1);

        debug!(
            "current task is to be preempted: {}, allow={}",
            curr.id_name(),
            can_preempt
        );
        if can_preempt {
            self.inner.resched(true);
        } else {
            curr.set_preempt_pending(true);
        }
    }

    /// Exit the current task with the specified exit code.
    /// This function will never return.
    pub fn exit_current(&mut self, exit_code: i32) -> ! {
        let curr = crate::current();
        debug!("task exit: {}, exit_code={}", curr.id_name(), exit_code);
        assert!(curr.is_running(), "task is not running: {:?}", curr.state());
        assert!(!curr.is_idle());
        if curr.is_init() {
            // Safety: it is called from `current_run_queue::<NoPreemptIrqSave>().exit_current(exit_code)`,
            // which disabled IRQs and preemption.
            unsafe {
                EXITED_TASKS.current_ref_mut_raw().clear();
            }
            axhal::misc::terminate();
        } else {
            curr.set_state(TaskState::Exited);

            // Notify the joiner task.
            curr.notify_exit(exit_code);

            // Safety: it is called from `current_run_queue::<NoPreemptIrqSave>().exit_current(exit_code)`,
            // which disabled IRQs and preemption.
            unsafe {
                // Push current task to the `EXITED_TASKS` list, which will be consumed by the GC task.
                EXITED_TASKS.current_ref_mut_raw().push_back(curr.clone());
                // Wake up the GC task to drop the exited tasks.
                WAIT_FOR_EXIT.current_ref_mut_raw().notify_one(false);
            }

            // Schedule to next task.
            self.inner.resched(false);
        }
        unreachable!("task exited!");
    }

    /// Block the current task, put current task into the wait queue and reschedule.
    /// Mark the state of current task as `Blocked`, set the `in_wait_queue` flag as true.
    /// Note:
    ///     1. The caller must hold the lock of the wait queue.
    ///     2. The caller must ensure that the current task is in the running state.
    ///     3. The caller must ensure that the current task is not the idle task.
    ///     4. The lock of the wait queue will be released explicitly after current task is pushed into it.
    pub fn blocked_resched(&mut self, mut wq_guard: WaitQueueGuard) {
        let curr = crate::current();
        assert!(curr.is_running());
        assert!(!curr.is_idle());
        // we must not block current task with preemption disabled.
        // Current expected preempt count is 2.
        // 1 for `NoPreemptIrqSave`, 1 for wait queue's `SpinNoIrq`.
        #[cfg(feature = "preempt")]
        assert!(curr.can_preempt(2));

        // Mark the task as blocked, this has to be done before adding it to the wait queue
        // while holding the lock of the wait queue.
        curr.set_state(TaskState::Blocked);
        curr.set_in_wait_queue(true);

        wq_guard.push_back(curr.clone());
        // Drop the lock of wait queue explictly.
        drop(wq_guard);

        // Current task's state has been changed to `Blocked` and added to the wait queue.
        // Note that the state may have been set as `Ready` in `unblock_task()`,
        // see `unblock_task()` for details.

        debug!("task block: {}", curr.id_name());
        self.inner.resched(false);
    }

    /// Unblock one task by inserting it into the run queue.
    /// If task's `on_cpu` flag is true,
    /// it will enter a loop until the task finishes its scheduling process.
    pub fn unblock_task(&mut self, task: AxTaskRef, resched: bool) {
        // Try to change the state of the task from `Blocked` to `Ready`,
        // if successful, put it into the run queue,
        // otherwise, the task is already unblocked by other cores.
        if task.transition_state(TaskState::Blocked, TaskState::Ready) {
            // Since now, the task to be unblocked is in the `Ready` state.

            // If the owning (remote) CPU is still in the middle of schedule() with
            // this task as prev, wait until it's done referencing the task.
            //
            // Pairs with the `clear_prev_task_on_cpu()`.
            //
            // This ensures that tasks getting woken will be fully ordered against
            // their previous state and preserve Program Order.
            //
            // Note:
            // 1. This should be placed after the judgement of `TaskState::Blocked,`,
            //    because the task may have been woken up by other cores.
            // 2. This can be placed in the front of `switch_to()`
            #[cfg(feature = "smp")]
            while task.on_cpu() {
                // Wait for the task to finish its scheduling process.
                core::hint::spin_loop();
            }
            #[cfg(feature = "smp")]
            assert!(!task.on_cpu());

            let cpu_id = self.inner.cpu_id;
            debug!("task unblock: {} on run_queue {}", task.id_name(), cpu_id);
            self.inner
                .scheduler
                .lock()
                .put_prev_task(task.clone(), resched); // TODO: priority

            // Note: when the task is unblocked on another CPU's run queue,
            // we just ingiore the `resched` flag.
            if resched && cpu_id == this_cpu_id() {
                #[cfg(feature = "preempt")]
                crate::current().set_preempt_pending(true);
            }
        }
    }

    #[cfg(feature = "irq")]
    pub fn sleep_until(&mut self, deadline: axhal::time::TimeValue) {
        let curr = crate::current();
        debug!("task sleep: {}, deadline={:?}", curr.id_name(), deadline);
        assert!(curr.is_running());
        assert!(!curr.is_idle());

        let now = axhal::time::wall_time();
        if now < deadline {
            crate::timers::set_alarm_wakeup(deadline, curr.clone());
            curr.set_state(TaskState::Blocked);
            self.inner.resched(false);
        }
    }
}

impl AxRunQueue {
    /// Create a new run queue for the specified CPU.
    /// The run queue is initialized with a per-CPU gc task in its scheduler.
    fn new(cpu_id: usize) -> Self {
        let gc_task = TaskInner::new(gc_entry, "gc".into(), axconfig::TASK_STACK_SIZE).into_arc();
        // gc task should be pinned to the current CPU.
        gc_task.set_cpumask(CpuMask::one_shot(cpu_id));

        let mut scheduler = Scheduler::new();
        scheduler.add_task(gc_task);
        Self {
            cpu_id,
            scheduler: SpinRaw::new(scheduler),
        }
    }

    /// Common reschedule subroutine. If `preempt`, keep current task's time
    /// slice, otherwise reset it.
    fn resched(&mut self, preempt: bool) {
        let prev = crate::current();
        if prev.is_running() {
            prev.set_state(TaskState::Ready);
            if !prev.is_idle() {
                self.scheduler.lock().put_prev_task(prev.clone(), preempt);
            }
        }

        let next = self
            .scheduler
            .lock()
            .pick_next_task()
            .unwrap_or_else(|| unsafe {
                // Safety: IRQs must be disabled at this time.
                IDLE_TASK.current_ref_raw().get_unchecked().clone()
            });
        assert!(
            next.is_ready(),
            "next {} is not ready: {:?}",
            next.id_name(),
            next.state()
        );
        self.switch_to(prev, next);
    }

    fn switch_to(&mut self, prev_task: CurrentTask, next_task: AxTaskRef) {
        // Make sure that IRQs are disabled by kernel guard or other means.
        #[cfg(all(not(test), feature = "irq"))] // Note: irq is faked under unit tests.
        assert!(
            !axhal::arch::irqs_enabled(),
            "IRQs must be disabled during scheduling"
        );
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

        // Claim the task as running, we do this before switching to it
        // such that any running task will have this set.
        #[cfg(feature = "smp")]
        next_task.set_on_cpu(true);

        unsafe {
            let prev_ctx_ptr = prev_task.ctx_mut_ptr();
            let next_ctx_ptr = next_task.ctx_mut_ptr();

            // Store the weak pointer of **prev_task** in **next_task**'s struct.
            #[cfg(feature = "smp")]
            next_task.set_prev_task(prev_task.as_task_ref());

            // The strong reference count of `prev_task` will be decremented by 1,
            // but won't be dropped until `gc_entry()` is called.
            assert!(Arc::strong_count(prev_task.as_task_ref()) > 1);
            assert!(Arc::strong_count(&next_task) >= 1);

            CurrentTask::set_current(prev_task, next_task);

            (*prev_ctx_ptr).switch_to(&*next_ctx_ptr);

            // Current it's **next_task** running on this CPU, clear the `prev_task`'s `on_cpu` field
            // to indicate that it has finished its scheduling process and no longer running on this CPU.
            #[cfg(feature = "smp")]
            crate::current().clear_prev_task_on_cpu();
        }
    }
}

fn gc_entry() {
    loop {
        // Drop all exited tasks and recycle resources.
        let n = EXITED_TASKS.with_current(|exited_tasks| exited_tasks.len());
        for _ in 0..n {
            // Do not do the slow drops in the critical section.
            let task = EXITED_TASKS.with_current(|exited_tasks| exited_tasks.pop_front());
            if let Some(task) = task {
                if Arc::strong_count(&task) == 1 {
                    // If I'm the last holder of the task, drop it immediately.
                    drop(task);
                } else {
                    // Otherwise (e.g, `switch_to` is not compeleted, held by the
                    // joiner, etc), push it back and wait for them to drop first.
                    EXITED_TASKS.with_current(|exited_tasks| exited_tasks.push_back(task));
                }
            }
        }
        // Note: we cannot block current task with preemption disabled,
        // use `current_ref_raw` to get the `WAIT_FOR_EXIT`'s reference here to avoid the use of `NoPreemptGuard`.
        // Since gc task is pinned to the current CPU, there is no affection if the gc task is preempted during the process.
        unsafe { WAIT_FOR_EXIT.current_ref_raw() }.wait();
    }
}

pub(crate) fn init() {
    let cpu_id = this_cpu_id();

    // Create the `idle` task (not current task).
    const IDLE_TASK_STACK_SIZE: usize = 4096;
    let idle_task = TaskInner::new(|| crate::run_idle(), "idle".into(), IDLE_TASK_STACK_SIZE);
    // idle task should be pinned to the current CPU.
    idle_task.set_cpumask(CpuMask::one_shot(cpu_id));
    IDLE_TASK.with_current(|i| {
        i.init_once(idle_task.into_arc());
    });

    // Put the subsequent execution into the `main` task.
    let main_task = TaskInner::new_init("main".into()).into_arc();
    main_task.set_state(TaskState::Running);
    unsafe { CurrentTask::init_current(main_task) }

    RUN_QUEUE.with_current(|rq| {
        rq.init_once(AxRunQueue::new(cpu_id));
    });
    unsafe {
        RUN_QUEUES[cpu_id].write(RUN_QUEUE.current_ref_mut_raw());
    }
}

pub(crate) fn init_secondary() {
    let cpu_id = this_cpu_id();

    // Put the subsequent execution into the `idle` task.
    let idle_task = TaskInner::new_init("idle".into()).into_arc();
    idle_task.set_state(TaskState::Running);
    IDLE_TASK.with_current(|i| {
        i.init_once(idle_task.clone());
    });
    unsafe { CurrentTask::init_current(idle_task) }

    RUN_QUEUE.with_current(|rq| {
        rq.init_once(AxRunQueue::new(cpu_id));
    });
    unsafe {
        RUN_QUEUES[cpu_id].write(RUN_QUEUE.current_ref_mut_raw());
    }
}

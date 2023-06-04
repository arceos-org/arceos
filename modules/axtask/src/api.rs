//! Task APIs for multi-task configuration.

pub(crate) use crate::run_queue::{AxRunQueue, RUN_QUEUE};

#[doc(cfg(feature = "multitask"))]
pub use crate::task::{CurrentTask, TaskId, TaskInner};
#[doc(cfg(feature = "multitask"))]
pub use crate::wait_queue::WaitQueue;

use alloc::string::String;

use crate::run_queue::LOAD_BALANCE_ARR;
use load_balance::BaseLoadBalance;

/// The reference type of a task.
pub type AxTaskRef = alloc::sync::Arc<AxTask>;

cfg_if::cfg_if! {
    if #[cfg(feature = "sched_fifo")] {
        pub(crate) type AxTask = scheduler::FifoTask<TaskInner>;
        pub(crate) type Scheduler = scheduler::FifoScheduler<TaskInner>;
    } else if #[cfg(feature = "sched_rr")] {
        const MAX_TIME_SLICE: usize = 5;
        pub(crate) type AxTask = scheduler::RRTask<TaskInner, MAX_TIME_SLICE>;
        pub(crate) type Scheduler = scheduler::RRScheduler<TaskInner, MAX_TIME_SLICE>;
    } else if #[cfg(feature = "sched_cfs")] {
        pub(crate) type AxTask = scheduler::CFSTask<TaskInner>;
        pub(crate) type Scheduler = scheduler::CFScheduler<TaskInner>;
    }
}

pub(crate) type LoadBalance = load_balance::BasicMethod;

#[cfg(feature = "preempt")]
struct KernelGuardIfImpl;

#[cfg(feature = "preempt")]
#[crate_interface::impl_interface]
impl kernel_guard::KernelGuardIf for KernelGuardIfImpl {
    fn disable_preempt() {
        if let Some(curr) = current_may_uninit() {
            curr.disable_preempt();
        }
    }

    fn enable_preempt() {
        if let Some(curr) = current_may_uninit() {
            curr.enable_preempt(true);
        }
    }
}

#[cfg(not(feature = "std"))]
#[def_interface]
/// get current time
pub trait LogMyTime {
    /// get current time
    fn current_cpu_id() -> Option<usize>;
}

use core::sync::atomic::Ordering;

use crate::INITED_CPUS;

/// Check whether all CPUs are inited
pub fn is_init_ok() -> bool {
    INITED_CPUS.load(Ordering::Acquire) == axconfig::SMP
}

use crate_interface::def_interface;

/// get current cpu id
pub fn get_current_cpu_id() -> usize {
    axhal::cpu::this_cpu_id()
}

/// Gets the current task, or returns [`None`] if the current task is not
/// initialized.
pub fn current_may_uninit() -> Option<CurrentTask> {
    CurrentTask::try_get()
}

/// Gets the current task.
///
/// # Panics
///
/// Panics if the current task is not initialized.
pub fn current() -> CurrentTask {
    CurrentTask::get()
}

/// Initializes the task scheduler (for the primary CPU).
pub fn init_scheduler() {
    info!("Initialize scheduling...");

    crate::run_queue::init();
    #[cfg(feature = "irq")]
    crate::timers::init();

    info!("  use {} scheduler.", Scheduler::scheduler_name());
    info!(
        "  use {} load balance manager.",
        LoadBalance::load_balance_name()
    );
}

/// Initializes the task scheduler for secondary CPUs.
pub fn init_scheduler_secondary() {
    crate::run_queue::init_secondary();
}

/// Handles periodic timer ticks for the task manager.
///
/// For example, advance scheduler states, checks timed events, etc.
#[cfg(feature = "irq")]
#[doc(cfg(feature = "irq"))]
pub fn on_timer_tick() {
    crate::timers::check_events();
    RUN_QUEUE[get_current_cpu_id()].scheduler_timer_tick()
}

/// Spawns a new task with the given parameters.
///
/// Returns the task reference.
pub fn spawn_raw<F>(f: F, name: String, stack_size: usize) -> AxTaskRef
where
    F: FnOnce() + Send + 'static,
{
    let task = TaskInner::new(f, name, stack_size);
    RUN_QUEUE[LOAD_BALANCE_ARR[get_current_cpu_id()].find_target_cpu(task.get_affinity())].add_task(task.clone());
    task
}

/// Spawns a new task with the default parameters.
///
/// The default task name is an empty string. The default task stack size is
/// [`axconfig::TASK_STACK_SIZE`].
///
/// Returns the task reference.
pub fn spawn<F>(f: F) -> AxTaskRef
where
    F: FnOnce() + Send + 'static,
{
    spawn_raw(f, "".into(), axconfig::TASK_STACK_SIZE)
}

/// set priority for current task.
/// In CFS, priority is the nice value, ranging from -20 to 19.
pub fn set_priority(prio: isize) -> bool {
    RUN_QUEUE[get_current_cpu_id()].set_priority(prio)
}

/// Current task gives up the CPU time voluntarily, and switches to another
/// ready task.
pub fn yield_now() {
    //info!("exit 233");
    RUN_QUEUE[get_current_cpu_id()].yield_current();
    //info!("exit 234");
    // TODO: 还没有把功能取出来的功能
}

/// Current task is going to sleep for the given duration.
///
/// If the feature `irq` is not enabled, it uses busy-wait instead.
pub fn sleep(dur: core::time::Duration) {
    sleep_until(axhal::time::current_time() + dur);
}

/// Current task is going to sleep, it will be woken up at the given deadline.
///
/// If the feature `irq` is not enabled, it uses busy-wait instead.
pub fn sleep_until(deadline: axhal::time::TimeValue) {
    //info!("exit 233");
    #[cfg(feature = "irq")]
    RUN_QUEUE[get_current_cpu_id()].sleep_until(deadline);
    //info!("exit 234");
    #[cfg(not(feature = "irq"))]
    axhal::time::busy_wait_until(deadline);
}

/// Exits the current task.
pub fn exit(exit_code: i32) -> ! {
    //info!("exit 233");
    RUN_QUEUE[get_current_cpu_id()].exit_current(exit_code)
}

/// The idle task routine.
///
/// It runs an infinite loop that keeps calling [`yield_now()`].
pub fn run_idle() -> ! {
    loop {
        yield_now();
        debug!("idle task: waiting for IRQs...");
        axhal::arch::wait_for_irqs();
    }
}

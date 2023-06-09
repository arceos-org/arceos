//! Various scheduler algorithms in a unified interface.
//!
//! Currently supported algorithms:
//!
//! - [`FifoScheduler`]: FIFO (First-In-First-Out) scheduler (cooperative).
//! - [`RRScheduler`]: Round-robin scheduler (preemptive).
//! - [`CFScheduler`]: Completely Fair Scheduler (preemptive).

#![cfg_attr(not(test), no_std)]
#![feature(const_mut_refs)]

mod cfs;
mod fifo;
mod mlfq;
mod rms;
mod round_robin;
mod sjf;
mod utils;
pub use utils::timer::current_ticks;

#[cfg(test)]
mod tests;

extern crate alloc;
extern crate crate_interface;

pub use cfs::{CFSTask, CFScheduler};
pub use fifo::{FifoScheduler, FifoTask};
pub use mlfq::{MLFQScheduler, MLFQTask};
pub use rms::{RMSTask, RMScheduler};
pub use round_robin::{RRScheduler, RRTask};
pub use sjf::{SJFScheduler, SJFTask};

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
    ///
    /// Returns `true` if the priority is set successfully.
    fn set_priority(&mut self, task: &Self::SchedItem, prio: isize) -> bool;

    /// Check if the scheduler is empty.
    fn is_empty(&self) -> bool;
}

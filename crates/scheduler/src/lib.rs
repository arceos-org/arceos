#![no_std]

mod fifo;
mod round_robin;

extern crate alloc;

use alloc::sync::Arc;

pub use fifo::{FifoSchedState, FifoScheduler};
pub use round_robin::{RRSchedState, RRScheduler};

pub trait Schedulable<S> {
    fn sched_state(&self) -> &S;

    fn update_sched_state<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&S) -> T;
}

pub trait BaseScheduler<S, T: Schedulable<S>> {
    fn init(&mut self);
    fn add_task(&mut self, task: Arc<T>);
    fn remove_task(&mut self, task: &Arc<T>);
    fn yield_task(&mut self, task: Arc<T>);
    fn pick_next_task(&mut self, prev: &Arc<T>) -> Option<&Arc<T>>;
    fn task_tick(&mut self, current: &Arc<T>) -> bool;
}

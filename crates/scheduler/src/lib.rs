#![cfg_attr(not(test), no_std)]
#![feature(const_trait_impl)]
#![feature(const_mut_refs)]

mod fifo;
mod round_robin;

#[cfg(test)]
mod tests;

extern crate alloc;

pub use fifo::{FifoScheduler, FifoTask};
pub use round_robin::{RRScheduler, RRTask};

pub trait BaseScheduler {
    type SchedItem;

    fn init(&mut self);
    fn add_task(&mut self, task: Self::SchedItem);
    fn remove_task(&mut self, task: &Self::SchedItem) -> Option<Self::SchedItem>;
    fn pick_next_task(&mut self) -> Option<Self::SchedItem>;
    fn put_prev_task(&mut self, prev: Self::SchedItem);
    fn task_tick(&mut self, current: &Self::SchedItem) -> bool;
}

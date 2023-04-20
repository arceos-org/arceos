#![cfg_attr(not(test), no_std)]
#![feature(const_trait_impl)]
#![feature(const_mut_refs)]

mod fifo;
mod round_robin;
mod cfs;
mod sjf;
mod mlfq;
mod rms;

mod utils;
pub use utils::timer::current_ticks;

#[cfg(test)]
mod tests;

extern crate alloc;
extern crate crate_interface;

pub use fifo::{FifoScheduler, FifoTask};
pub use round_robin::{RRScheduler, RRTask};
pub use cfs::{CFScheduler, CFTask};
pub use sjf::{SJFScheduler, SJFTask};
pub use mlfq::{MLFQScheduler, MLFQTask};
pub use rms::{RMScheduler, RMSTask};


pub trait BaseScheduler {
    type SchedItem;

    fn init(&mut self);
    fn add_task(&mut self, task: Self::SchedItem);
    fn remove_task(&mut self, task: &Self::SchedItem) -> Option<Self::SchedItem>;
    fn pick_next_task(&mut self) -> Option<Self::SchedItem>;
    fn put_prev_task(&mut self, prev: Self::SchedItem, preempt: bool);
    fn task_tick(&mut self, current: &Self::SchedItem) -> bool;
}

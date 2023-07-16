//! Monolithic 存放为了实现宏内核架构而改造的task结构

pub mod task;

mod copy;

mod stat;
#[cfg(feature = "signal")]
pub use stat::SignalCaller;
pub mod run_queue;

pub use run_queue::{EXITED_TASKS, IDLE_TASK, RUN_QUEUE};

#![cfg_attr(not(test), no_std)]
#![feature(drain_filter)]

use axhal::arch::TaskContext;
use axtask::{current, task::CurrentTask};
extern crate alloc;

pub mod flags;
pub mod mem;
pub mod process;
pub mod signal;

/// 开始进行调度，我们先执行gc任务，通过gc任务逐个执行并收集RUN_QUEUE中的任务
/// 所以先切换到gc对应的任务上下文即可
pub fn start_schedule() {
    // 若是第一次执行任务，curr应当为gc
    let curr: CurrentTask = current();
    #[cfg(feature = "preempt")]
    curr.set_preempt_pending(false);
    curr.set_state_running();
    unsafe {
        let prev_ctx_ptr = TaskContext::new_empty();
        let next_ctx_ptr = curr.ctx_mut_ptr();
        // The strong reference count of `prev_task` will be decremented by 1,
        // but won't be dropped until `gc_entry()` is called.
        (*prev_ctx_ptr).switch_to(&*next_ctx_ptr);
    }
}

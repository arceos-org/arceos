#![cfg_attr(not(test), no_std)]
#![feature(drain_filter)]

use axconfig::TASK_STACK_SIZE;
use axhal::arch::TaskContext;
use axlog::info;
use axtask::{
    current,
    task::{CurrentTask, TaskInner},
    RUN_QUEUE,
};
use process::{current_process, Process, KERNEL_PROCESS_ID};
extern crate alloc;
pub mod flags;
pub mod process;
pub mod fd_manager;
mod test;
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

/// 当从内核态到用户态时，统计对应进程的时间信息
pub fn time_stat_from_kernel_to_user() {
    let curr_task = current();
    curr_task.time_stat_from_kernel_to_user();
}

/// 当从用户态到内核态时，统计对应进程的时间信息
pub fn time_stat_from_user_to_kernel() {
    let curr_task = current();
    curr_task.time_stat_from_user_to_kernel();
}

/// 统计时间输出
/// (用户态秒，用户态微妙，内核态秒，内核态微妙)
pub fn time_stat_output() -> (usize, usize, usize, usize) {
    let curr_task = current();
    curr_task.time_stat_output()
}

/// 初始化运行应用程序
pub fn init_tasks() {
    #[cfg(feature = "test")]
    {
        let test_scheduler = TaskInner::new(
            || test::run_testcases(),
            "test_scheduler",
            TASK_STACK_SIZE,
            KERNEL_PROCESS_ID,
            0,
        );
        // info!("test!");
        RUN_QUEUE.lock().add_task(test_scheduler);
    }
    #[cfg(not(feature = "test"))]
    {
        // 之后也许会添加一个专门的初始化进程
        let main_task = Process::new("waitpid");
        RUN_QUEUE.lock().add_task(main_task);
    }
}

use core::ops::Deref;
extern crate alloc;
use alloc::sync::Arc;
use axhal::KERNEL_PROCESS_ID;
use axlog::info;
use axtask::{current, yield_now, TaskId, TaskState, IDLE_TASK, RUN_QUEUE};

use crate::process::{Process, PID2PC, TID2TASK};

/// 初始化内核调度进程
pub fn init_kernel_process() {
    let kernel_process = Arc::new(Process::new(TaskId::new().as_u64(), 0));

    axtask::init_scheduler();
    kernel_process.tasks.lock().push(Arc::clone(unsafe {
        &IDLE_TASK.current_ref_raw().get_unchecked()
    }));
    PID2PC.lock().insert(kernel_process.pid(), kernel_process);
}

pub fn current_process() -> Arc<Process> {
    let current_task = current();

    let current_process = Arc::clone(PID2PC.lock().get(&current_task.get_process_id()).unwrap());

    current_process
}

/// 退出当前任务
pub fn exit_current_task(exit_code: i32) -> ! {
    let process = current_process();
    let current_task = current();

    let curr_id = current_task.id().as_u64();

    info!("exit task id {} with code {}", curr_id, exit_code);

    if current_task.is_leader() {
        loop {
            let mut all_exited = true;

            for task in process.tasks.lock().deref() {
                if !task.is_leader() && task.state() != TaskState::Exited {
                    all_exited = false;
                    break;
                }
            }
            if !all_exited {
                yield_now();
            } else {
                break;
            }
        }

        TID2TASK.lock().remove(&curr_id);

        process.set_exit_code(exit_code);

        process.set_zombie(true);

        process.tasks.lock().clear();
        let mut pid2pc = PID2PC.lock();
        let kernel_process = pid2pc.get(&KERNEL_PROCESS_ID).unwrap();
        // 将子进程交给idle进程
        for child in process.children.lock().deref() {
            child.set_parent(KERNEL_PROCESS_ID);
            kernel_process.children.lock().push(Arc::clone(&child));
        }
        pid2pc.remove(&process.pid());
        drop(pid2pc);
        drop(process);
    } else {
        TID2TASK.lock().remove(&curr_id);
        drop(process);
    }
    RUN_QUEUE.lock().exit_current(exit_code);
}

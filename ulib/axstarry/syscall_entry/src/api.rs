extern crate alloc;

use alloc::string::String;
use alloc::sync::Arc;
use axhal::{
    arch::{flush_tlb, write_page_table_root},
    KERNEL_PROCESS_ID,
};
use axprocess::{yield_now_task, PID2PC};
use axruntime::KERNEL_PAGE_TABLE;
use axtask::{TaskId, EXITED_TASKS};
use syscall_utils::init_current_dir;

/// 释放所有非内核进程
pub fn recycle_user_process() {
    let kernel_process = Arc::clone(PID2PC.lock().get(&KERNEL_PROCESS_ID).unwrap());

    loop {
        let mut all_finished = true;
        let pid2pc = PID2PC.lock();
        for children in kernel_process.children.lock().iter() {
            if children.pid() == KERNEL_PROCESS_ID {
                continue;
            }
            if pid2pc.contains_key(&children.pid()) {
                all_finished = false;
                break;
            }
            // remove the exited process
            kernel_process
                .children
                .lock()
                .retain(|x| x.pid() != children.pid());
        }
        drop(pid2pc);
        if all_finished {
            break;
        }
        yield_now_task();
    }
    TaskId::clear();
    unsafe {
        write_page_table_root(KERNEL_PAGE_TABLE.root_paddr());
        flush_tlb(None);
    };
    EXITED_TASKS.lock().clear();
    init_current_dir();
}

pub fn println(s: &str) {
    axlog::ax_println!("{}", s);
}

pub fn read_file(_path: &str) -> Option<String> {
    #[cfg(feature = "syscall_fs")]
    {
        syscall_fs::read_file(_path)
    }
    #[cfg(not(feature = "syscall_fs"))]
    {
        None
    }
}

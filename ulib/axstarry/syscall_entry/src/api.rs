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
        let pid2pc = PID2PC.lock();

        kernel_process
            .children
            .lock()
            .retain(|x| x.pid() == KERNEL_PROCESS_ID || pid2pc.contains_key(&x.pid()));
        let all_finished = pid2pc.len() == 1;
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

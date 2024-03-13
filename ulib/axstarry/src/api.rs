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

use axerrno::AxResult;
use axfs::api::{File, OpenFlags};

/// 若使用多次new file打开同名文件，那么不同new file之间读写指针不共享，但是修改的内容是共享的
pub fn new_file(path: &str, flags: &OpenFlags) -> AxResult<File> {
    let mut file = File::options();
    file.read(flags.readable());
    file.write(flags.writable());
    file.create(flags.creatable());
    file.create_new(flags.new_creatable());
    file.open(path)
}
/// 在完成一次系统调用之后，恢复全局目录
pub fn init_current_dir() {
    axfs::api::set_current_dir("/").expect("reset current dir failed");
}

/// Flags for opening a file
pub type FileFlags = OpenFlags;

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

/// To print a string
pub fn println(s: &str) {
    axlog::ax_println!("{}", s);
}

/// To read a file with the given path
pub fn read_file(_path: &str) -> Option<String> {
    #[cfg(feature = "syscall_fs")]
    {
        axfs::api::read_to_string(_path).ok()
    }
    #[cfg(not(feature = "syscall_fs"))]
    {
        None
    }
}

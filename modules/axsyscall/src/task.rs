use axtask::{current, mem::memory_set::get_app_data, process::PID2PC};
extern crate alloc;
use alloc::{vec::Vec, sync::Arc};
/// 处理与任务（线程）有关的系统调用

pub fn syscall_exit() -> isize {
    axlog::info!("Syscall to exit!");
    axtask::exit(0)
}

pub fn syscall_exec(path: *const u8, mut args: *const usize) -> isize {
    let curr = current();
    let pid2pc_inner = PID2PC.lock();
    let curr_process = Arc::clone(&pid2pc_inner.get(&curr.get_process_id()).unwrap());
    drop(pid2pc_inner);
    let inner = curr_process.inner.lock();
    let path = inner.memory_set.lock().translate_str(path);
    axlog::info!("path: {}", path);
    axlog::info!("Syscall to exec {}", path);
    let mut args_vec = Vec::new();
    loop {
        let args_str_ptr = unsafe { *args };
        if args_str_ptr == 0 {
            break;
        }
        args_vec.push(inner.memory_set.lock().translate_str(args_str_ptr as *const u8));
        unsafe {
            args = args.add(1);
        }
    }
    drop(inner);
    let elf_data = get_app_data(&path);
    let argc = args_vec.len();
    curr_process.exec(elf_data, args_vec);
    argc as isize
}

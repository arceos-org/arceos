use crate::{current, mem::memory_set::get_app_data};
use alloc::vec::Vec;
/// 处理与任务（线程）有关的系统调用

pub fn syscall_exit() -> isize {
    axlog::info!("Syscall to exit!");
    crate::exit(0)
}

pub fn syscall_exec(path: *const u8, mut args: *const usize) -> isize {
    let curr = current();
    let inner = curr.process.inner.lock();
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
    curr.process.exec(elf_data, args_vec);
    argc as isize
}

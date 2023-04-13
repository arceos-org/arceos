/// 处理与任务（线程）有关的系统调用
use axlog;

pub fn syscall_exit() -> isize {
    axlog::info!("Syscall to exit!");
    crate::exit(0)
}

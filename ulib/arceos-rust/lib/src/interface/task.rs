#[cfg(feature = "multitask")]
use crate::interface::thread::take_task;
use arceos_api::modules::axlog::{ax_println, info};
use arceos_api::sys::ax_terminate;

#[unsafe(no_mangle)]
pub fn sys_abort() -> ! {
    info!("called sys_abort");
    #[cfg(feature = "multitask")]
    take_task(arceos_api::task::ax_current_task_id());
    ax_terminate()
}

#[unsafe(no_mangle)]
pub fn sys_exit(code: i32) -> ! {
    info!("called sys_exit with code {}", code);
    #[cfg(feature = "multitask")]
    take_task(arceos_api::task::ax_current_task_id());
    ax_println!("[ArceOS] Process exited with code {}", code);
    ax_terminate()
}

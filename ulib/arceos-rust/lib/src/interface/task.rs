use arceos_api::modules::axlog::{ax_println, info};
use arceos_api::sys::ax_terminate;

#[unsafe(no_mangle)]
pub fn sys_abort() -> ! {
    info!("called sys_abort");
    ax_terminate()
}

#[unsafe(no_mangle)]
pub fn sys_exit(code: i32) -> ! {
    info!("called sys_exit with code {}", code);
    ax_println!("[ArceOS] Process exited with code {}", code);
    ax_terminate()
}

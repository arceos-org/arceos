#![no_std]
extern crate alloc;

mod syscall;
mod interface;

use arceos_api::modules::axlog::{debug, info};
pub use axruntime;

unsafe extern "C" {
    fn runtime_entry(argc: i32, argv: *const *const u8, env: *const *const u8) -> !;
}

#[unsafe(no_mangle)]
pub fn __app_main() {
    info!("Starting application...");
    // call the runtime entry point with zeroed arguments
    const ARGC: i32 = 1;
    const NAME: &[u8; 9] = b"app_name\0";
    let argv: [*const u8; 2] = [NAME.as_ptr(), core::ptr::null()];
    let env: [*const u8; 1] = [core::ptr::null()];
    debug!("address of runtime_entry: {:p}", runtime_entry as *const ());
    unsafe {
        runtime_entry(ARGC, argv.as_ptr(), env.as_ptr());
    }
}

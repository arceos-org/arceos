#![no_std]
extern crate alloc;

mod interface;

use arceos_api::modules::axlog::{debug, info};
use axerrno::LinuxError;
pub use axruntime;

unsafe extern "C" {
    fn runtime_entry(argc: i32, argv: *const *const u8, env: *const *const u8) -> !;
}

pub(crate) fn err(error: LinuxError) -> i32 {
    -(error as i32)
}

#[unsafe(no_mangle)]
pub fn __app_main() {
    // keep `linkme` symbols from being optimized out
    #[cfg(feature = "irq")]
    {
        use arceos_api::modules::axhal::irq::irq_handler;
        core::hint::black_box(irq_handler as *const ());
    }
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

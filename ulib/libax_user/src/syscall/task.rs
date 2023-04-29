extern crate alloc;
use alloc::boxed::Box;

use crate::sys_number::{SYS_SBRK, SYS_SLEEP, SYS_SPAWN, SYS_YIELD};

use super::sys_number::SYS_EXIT;

pub fn exit(exitcode: usize) -> ! {
    crate::syscall(SYS_EXIT, [exitcode, 0, 0, 0, 0, 0]);
    unreachable!("program already terminated")
}

pub fn spawn_fn(f: fn()) {
    crate::syscall(SYS_SPAWN, [f as usize, 0, 0, 0, 0, 0]);
}

pub fn yield_now() {
    crate::syscall(SYS_YIELD, [0, 0, 0, 0, 0, 0]);
}

pub fn sleep(t: core::time::Duration) {
    crate::syscall(
        SYS_SLEEP,
        [t.as_secs() as usize, t.subsec_nanos() as usize, 0, 0, 0, 0],
    );
}

type BoxedFn = Box<dyn FnOnce() + Sync + 'static>;

fn child_task_start(arg: usize) {
    let run_fn = unsafe { alloc::boxed::Box::from_raw(arg as *mut BoxedFn) };
    run_fn();
    exit(0);
}

// reference: https://doc.rust-lang.org/src/std/sys/unix/thread.rs.html
pub fn spawn<F>(f: F)
where
    F: FnOnce() + Sync + 'static,
{
    let run_fn: BoxedFn = Box::new(f);
    let run_fn_raw = Box::into_raw(Box::new(run_fn));
    crate::syscall(
        SYS_SPAWN,
        [
            child_task_start as usize,
            run_fn_raw as *const u8 as usize,
            0,
            0,
            0,
            0,
        ],
    );
}

pub fn sbrk(size: isize) -> isize {
    crate::syscall(SYS_SBRK, [size as usize, 0, 0, 0, 0, 0])
}

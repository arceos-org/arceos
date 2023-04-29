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

pub fn spawn<F>(_f: F)
where
    F: FnOnce() + Sync + 'static,
{
    unimplemented!();
}

pub fn sbrk(size: isize) -> isize {
    crate::syscall(SYS_SBRK, [size as usize, 0, 0, 0, 0, 0])
}

use alloc::format;
use alloc::vec::Vec;
use arceos_api::modules::axlog::warn;
use arceos_api::task::{AxTaskHandle, ax_spawn, ax_wait_for_exit, ax_yield_now};
use arceos_posix_api::ctypes::Tid;
use core::sync::atomic::{AtomicU64, Ordering};
use kspin::SpinNoIrq;

const DEFAULT_STACK_SIZE: usize = arceos_api::config::TASK_STACK_SIZE;

static TASK_TABLE: SpinNoIrq<Vec<(u64, AxTaskHandle)>> = SpinNoIrq::new(Vec::new());
static NEXT_TID: AtomicU64 = AtomicU64::new(1);

fn insert_task(handle: AxTaskHandle) -> u64 {
    let tid = NEXT_TID.fetch_add(1, Ordering::Relaxed);
    let mut guard = TASK_TABLE.lock();
    guard.push((tid, handle));
    tid
}

fn take_task(tid: u64) -> Option<AxTaskHandle> {
    let mut guard = TASK_TABLE.lock();
    if let Some(index) = guard.iter().position(|(id, _)| *id == tid) {
        Some(guard.swap_remove(index).1)
    } else {
        None
    }
}

/// spawn a new thread with user-specified stack size
///
/// spawn2() starts a new thread. The new thread starts execution
/// by invoking `func(usize)`; `arg` is passed as the argument
/// to `func`. `prio` defines the priority of the new thread,
/// which can be between `LOW_PRIO` and `HIGH_PRIO`.
/// `core_id` defines the core, where the thread is located.
/// A negative value give the operating system the possibility
/// to select the core by its own.
/// In contrast to spawn(), spawn2() is able to define the
/// stack size.
#[cfg(feature = "multitask")]
#[unsafe(no_mangle)]
pub fn sys_spawn2(
    func: extern "C" fn(usize),
    arg: usize,
    _prio: u8,
    stack_size: usize,
    _core_id: isize,
) -> Tid {
    let actual_stack = if stack_size == 0 { DEFAULT_STACK_SIZE } else { stack_size };
    let name = format!("hermit-thread-{}", NEXT_TID.load(Ordering::Relaxed));
    let handle = ax_spawn(
        move || {
            func(arg);
        },
        name,
        actual_stack,
    );
    insert_task(handle) as Tid
}

/// Wait for a thread to finish.
#[cfg(feature = "multitask")]
#[unsafe(no_mangle)]
pub fn sys_join(tid: Tid) -> i32 {
    match take_task(tid as u64) {
        Some(handle) => ax_wait_for_exit(handle).unwrap_or(0),
        None => {
            warn!("[sys_join] Unknown tid {}", tid);
            -1
        }
    }
}

/// Yield the current thread.
#[cfg(feature = "multitask")]
#[unsafe(no_mangle)]
pub fn sys_yield() {
    ax_yield_now();
}

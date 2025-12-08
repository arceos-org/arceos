use alloc::collections::BTreeMap;
use alloc::format;
use arceos_api::modules::axsync::Mutex;
use arceos_api::task::{ax_spawn, ax_wait_for_exit, ax_yield_now, AxTaskHandle};
use arceos_posix_api::ctypes::Tid;
use core::sync::atomic::{AtomicU64, Ordering};
use log::{info, warn};

const DEFAULT_STACK_SIZE: usize = arceos_api::config::TASK_STACK_SIZE;

static NEXT_TASK_NAME: AtomicU64 = AtomicU64::new(1);
static TASK_TABLE: Mutex<BTreeMap<u64, AxTaskHandle>> = Mutex::new(BTreeMap::new());

pub fn insert_task(handle: AxTaskHandle) -> u64 {
    let tid = handle.id();
    TASK_TABLE.lock().insert(tid, handle);
    tid
}

pub fn take_task(tid: u64) -> Option<AxTaskHandle> {
    TASK_TABLE.lock().remove(&tid)
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
#[unsafe(no_mangle)]
pub fn sys_spawn2(
    func: extern "C" fn(usize),
    arg: usize,
    _prio: u8,
    stack_size: usize,
    _core_id: isize,
) -> Tid {
    let actual_stack = if stack_size == 0 {
        DEFAULT_STACK_SIZE
    } else {
        stack_size
    };
    let name = format!(
        "hermit-thread-{}",
        NEXT_TASK_NAME.fetch_add(1, Ordering::Relaxed)
    );
    let handle = ax_spawn(
        move || {
            func(arg);
        },
        name,
        actual_stack,
    );
    info!(
        "called sys_spawn2: func={:?}, arg={}, stack_size={}",
        func as *const (), arg, actual_stack
    );
    info!("created new thread with tid {}", handle.id());
    insert_task(handle) as Tid
}

/// Wait for a thread to finish.
#[unsafe(no_mangle)]
pub fn sys_join(tid: Tid) -> i32 {
    match take_task(tid as u64) {
        Some(handle) => ax_wait_for_exit(handle),
        None => {
            warn!("[sys_join] Unknown tid {}", tid);
            -1
        }
    }
}

/// Yield the current thread.
#[unsafe(no_mangle)]
pub fn sys_yield() {
    ax_yield_now();
}

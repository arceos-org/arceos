use alloc::string::ToString;
use arceos_api::task::ax_spawn;
use arceos_posix_api::ctypes::Tid;

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
    let task = ax_spawn(
        move || {
            func(arg);
        },
        (func as usize).to_string(),
        stack_size,
    );
    task.id() as _
}

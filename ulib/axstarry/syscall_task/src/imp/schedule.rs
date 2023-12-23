//! 支持与任务调度相关的 syscall
extern crate alloc;
use alloc::sync::Arc;
use axconfig::SMP;
use axhal::mem::VirtAddr;
use axprocess::{current_process, current_task, PID2PC, TID2TASK};

// #[cfg(feature = "signal")]
use axtask::{SchedPolicy, SchedStatus};

use syscall_utils::{SchedParam, SyscallError, SyscallResult};

/// 获取对应任务的CPU适配集
///
/// 若pid是进程ID，则获取对应的进程的主线程的信息
///
/// 若pid是线程ID，则获取对应线程信息
///
/// 若pid为0，则获取当前运行任务的信息
///
/// mask为即将写入的cpu set的地址指针
pub fn syscall_sched_getaffinity(
    pid: usize,
    cpu_set_size: usize,
    mask: *mut usize,
) -> SyscallResult {
    // let task: LazyInit<AxTaskRef> = LazyInit::new();
    let tid2task = TID2TASK.lock();
    let pid2task = PID2PC.lock();
    let pid = pid as u64;
    let task = if tid2task.contains_key(&pid) {
        Arc::clone(&tid2task.get(&pid).unwrap())
    } else if pid2task.contains_key(&pid) {
        let process = pid2task.get(&pid).unwrap();

        process
            .tasks
            .lock()
            .iter()
            .find(|task| task.is_leader())
            .map(|task| Arc::clone(task))
            .unwrap()
    } else if pid == 0 {
        Arc::clone(current_task().as_task_ref())
    } else {
        // 找不到对应任务
        return Err(SyscallError::ESRCH);
    };

    drop(pid2task);
    drop(tid2task);

    let process = current_process();
    if process
        .manual_alloc_for_lazy(VirtAddr::from(mask as usize))
        .is_err()
    {
        return Err(SyscallError::EFAULT);
    }
    let cpu_set = task.get_cpu_set();
    let mut prev_mask = unsafe { *mask };
    let len = SMP.min(cpu_set_size * 4);
    prev_mask &= !((1 << len) - 1);
    prev_mask &= cpu_set & ((1 << len) - 1);
    unsafe {
        *mask = prev_mask;
    }
    // 返回成功填充的缓冲区的长度
    Ok(SMP as isize)
}

#[allow(unused)]
pub fn syscall_sched_setaffinity(
    pid: usize,
    cpu_set_size: usize,
    mask: *const usize,
) -> SyscallResult {
    let tid2task = TID2TASK.lock();
    let pid2task = PID2PC.lock();
    let pid = pid as u64;
    let task = if tid2task.contains_key(&pid) {
        Arc::clone(&tid2task.get(&pid).unwrap())
    } else if pid2task.contains_key(&pid) {
        let process = pid2task.get(&pid).unwrap();

        process
            .tasks
            .lock()
            .iter()
            .find(|task| task.is_leader())
            .map(|task| Arc::clone(task))
            .unwrap()
    } else if pid == 0 {
        Arc::clone(current_task().as_task_ref())
    } else {
        // 找不到对应任务
        return Err(SyscallError::ESRCH);
    };

    drop(pid2task);
    drop(tid2task);

    let process = current_process();
    if process
        .manual_alloc_for_lazy(VirtAddr::from(mask as usize))
        .is_err()
    {
        return Err(SyscallError::EFAULT);
    }

    let mask = unsafe { *mask };

    task.set_cpu_set(mask, cpu_set_size);

    Ok(0)
}

pub fn syscall_sched_setscheduler(
    pid: usize,
    policy: usize,
    param: *const SchedParam,
) -> SyscallResult {
    if (pid as isize) < 0 || param.is_null() {
        return Err(SyscallError::EINVAL);
    }

    let tid2task = TID2TASK.lock();
    let pid2task = PID2PC.lock();
    let pid = pid as u64;
    let task = if tid2task.contains_key(&pid) {
        Arc::clone(&tid2task.get(&pid).unwrap())
    } else if pid2task.contains_key(&pid) {
        let process = pid2task.get(&pid).unwrap();

        process
            .tasks
            .lock()
            .iter()
            .find(|task| task.is_leader())
            .map(|task| Arc::clone(task))
            .unwrap()
    } else if pid == 0 {
        Arc::clone(current_task().as_task_ref())
    } else {
        // 找不到对应任务
        return Err(SyscallError::ESRCH);
    };

    drop(pid2task);
    drop(tid2task);

    let process = current_process();
    if process
        .manual_alloc_for_lazy(VirtAddr::from(param as usize))
        .is_err()
    {
        return Err(SyscallError::EFAULT);
    }

    let param = unsafe { *param };
    let policy = SchedPolicy::from(policy);
    if policy == SchedPolicy::SCHED_UNKNOWN {
        return Err(SyscallError::EINVAL);
    }
    if policy == SchedPolicy::SCHED_OTHER
        || policy == SchedPolicy::SCHED_BATCH
        || policy == SchedPolicy::SCHED_IDLE
    {
        if param.sched_priority != 0 {
            return Err(SyscallError::EINVAL);
        }
    } else {
        if param.sched_priority < 1 || param.sched_priority > 99 {
            return Err(SyscallError::EINVAL);
        }
    }

    task.set_sched_status(SchedStatus {
        policy,
        priority: param.sched_priority,
    });

    Ok(0)
}

pub fn syscall_sched_getscheduler(pid: usize) -> SyscallResult {
    if (pid as isize) < 0 {
        return Err(SyscallError::EINVAL);
    }

    let tid2task = TID2TASK.lock();
    let pid2task = PID2PC.lock();
    let pid = pid as u64;
    let task = if tid2task.contains_key(&pid) {
        Arc::clone(&tid2task.get(&pid).unwrap())
    } else if pid2task.contains_key(&pid) {
        let process = pid2task.get(&pid).unwrap();

        process
            .tasks
            .lock()
            .iter()
            .find(|task| task.is_leader())
            .map(|task| Arc::clone(task))
            .unwrap()
    } else if pid == 0 {
        Arc::clone(current_task().as_task_ref())
    } else {
        // 找不到对应任务
        return Err(SyscallError::ESRCH);
    };

    drop(pid2task);
    drop(tid2task);

    let policy: isize = task.get_sched_status().policy.into();
    Ok(policy)
}
